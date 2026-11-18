//! URL-normalized import maps for the resource-only module loader.
use serde_json::Value;
use std::collections::BTreeMap;
use url::Url;

type SpecifierMap = BTreeMap<String, Option<Url>>;
#[derive(Default)]
pub(crate) struct ImportMap {
    imports: SpecifierMap,
    scopes: BTreeMap<String, SpecifierMap>,
}

fn url_like(value: &str, base: &Url) -> Option<Url> {
    if value.starts_with('/') || value.starts_with("./") || value.starts_with("../") {
        base.join(value).ok()
    } else {
        Url::parse(value).ok()
    }
}

fn parse_specifiers(value: &Value, base: &Url) -> Result<SpecifierMap, String> {
    let object = value
        .as_object()
        .ok_or("Import map specifiers must be an object")?;
    Ok(object
        .iter()
        .filter(|(key, _)| !key.is_empty())
        .map(|(key, value)| {
            let key = url_like(key, base)
                .map(|url| url.to_string())
                .unwrap_or_else(|| key.clone());
            let address = value
                .as_str()
                .and_then(|value| url_like(value, base))
                .filter(|address| !key.ends_with('/') || address.as_str().ends_with('/'));
            (key, address)
        })
        .collect())
}

fn merge(target: &mut SpecifierMap, source: SpecifierMap) {
    for (key, value) in source {
        target.entry(key).or_insert(value);
    }
}

impl ImportMap {
    pub fn add(&mut self, json: &str, base: &Url) -> Result<(), String> {
        let value: Value = serde_json::from_str(json)
            .map_err(|error| format!("Invalid import map JSON: {error}"))?;
        let object = value.as_object().ok_or("Import map must be an object")?;
        // Parse completely before merging, so invalid maps cannot partially apply.
        let imports = object
            .get("imports")
            .map(|value| parse_specifiers(value, base))
            .transpose()?
            .unwrap_or_default();
        let mut scopes = BTreeMap::new();
        if let Some(value) = object.get("scopes") {
            for (scope, specifiers) in value
                .as_object()
                .ok_or("Import map scopes must be an object")?
            {
                let entries = parse_specifiers(specifiers, base)?;
                if let Ok(url) = base.join(scope) {
                    scopes.insert(url.to_string(), entries);
                }
            }
        }
        merge(&mut self.imports, imports);
        for (scope, entries) in scopes {
            merge(self.scopes.entry(scope).or_default(), entries);
        }
        Ok(())
    }

    pub fn resolve(&self, specifier: &str, base: &Url) -> Result<Url, String> {
        let url = url_like(specifier, base);
        let normalized = url.as_ref().map(Url::as_str).unwrap_or(specifier);
        for (scope, map) in self.scopes.iter().rev() {
            if (base.as_str() == scope
                || (scope.ends_with('/') && base.as_str().starts_with(scope)))
                && let Some(result) = resolve_map(map, normalized)
            {
                return result;
            }
        }
        if let Some(result) = resolve_map(&self.imports, normalized) {
            return result;
        }
        url.ok_or_else(|| format!("Unmapped bare module specifier: {specifier}"))
    }
}

fn resolve_map(map: &SpecifierMap, specifier: &str) -> Option<Result<Url, String>> {
    for (key, address) in map.iter().rev() {
        if specifier == key {
            return Some(
                address
                    .clone()
                    .ok_or_else(|| format!("Import map blocks {specifier}")),
            );
        }
        if key.ends_with('/') && specifier.starts_with(key) {
            return Some((|| {
                let base = address
                    .as_ref()
                    .ok_or_else(|| format!("Import map blocks {specifier}"))?;
                let resolved = base
                    .join(&specifier[key.len()..])
                    .map_err(|_| format!("Invalid mapped module: {specifier}"))?;
                if !resolved.as_str().starts_with(base.as_str()) {
                    return Err(format!("Mapped module escapes its prefix: {specifier}"));
                }
                Ok(resolved)
            })());
        }
    }
    None
}
