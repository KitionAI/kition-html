//! Generated counter text, resolved in document order before layout construction.
use std::collections::HashMap;

use blitz_traits::node_id::NodeId;
use style::{
    counter_style::CounterStyle,
    properties::ComputedValues,
    values::computed::{Content, ContentItem},
};
use stylo_taffy::CopyStyleGetters;

use super::damage::ALL_DAMAGE;
use crate::BaseDocument;

// Index 2 denotes the real element; 1 and 0 denote ::before and ::after.
type Origin = (NodeId, usize);
#[derive(Clone)]
struct Counter {
    name: String,
    origin: Origin,
    parent: Option<Origin>,
    value: i32,
}

#[derive(Default)]
struct Resolver {
    previous: Vec<Counter>,
    text: HashMap<Origin, String>,
}

impl Resolver {
    fn inherit(&self, parent: &[Counter], sibling: &[Counter]) -> Vec<Counter> {
        let mut result = parent.to_vec();
        for counter in sibling {
            if !result.iter().any(|c| c.name == counter.name) {
                result.push(counter.clone());
            }
        }
        for counter in &mut result {
            if let Some(source) = self
                .previous
                .iter()
                .find(|c| c.name == counter.name && c.origin == counter.origin)
            {
                counter.value = source.value;
            }
        }
        result
    }

    fn instantiate(
        counters: &mut Vec<Counter>,
        name: &str,
        origin: Origin,
        parent: Option<Origin>,
        value: i32,
    ) {
        if let Some(index) = counters.iter().rposition(|c| c.name == name) {
            if counters[index].origin == origin || counters[index].parent == parent {
                counters.remove(index);
            }
        }
        counters.push(Counter {
            name: name.into(),
            origin,
            parent,
            value,
        });
    }

    fn ensure(
        counters: &mut Vec<Counter>,
        name: &str,
        origin: Origin,
        parent: Option<Origin>,
    ) -> usize {
        if let Some(index) = counters.iter().rposition(|c| c.name == name) {
            return index;
        }
        Self::instantiate(counters, name, origin, parent, 0);
        counters.len() - 1
    }

    fn apply(
        style: &ComputedValues,
        counters: &mut Vec<Counter>,
        origin: Origin,
        parent: Option<Origin>,
    ) {
        let properties = style.get_counters();
        for pair in properties.counter_reset.iter() {
            Self::instantiate(counters, pair.name.0.as_ref(), origin, parent, pair.value);
        }
        for pair in properties.counter_increment.iter() {
            let index = Self::ensure(counters, pair.name.0.as_ref(), origin, parent);
            counters[index].value = counters[index].value.saturating_add(pair.value);
        }
    }

    fn pseudo(
        &mut self,
        doc: &BaseDocument,
        node: NodeId,
        idx: usize,
        parent: &[Counter],
        sibling: &[Counter],
    ) -> Option<Vec<Counter>> {
        let style_data = doc.nodes[node]
            .try_stylo_element_data()
            .and_then(|s| s.get())?;
        let style = style_data.styles.pseudos.as_array()[idx].as_ref()?;
        if style.clone_display().is_none() {
            return None;
        }
        let Content::Items(data) = &style.get_counters().content else {
            return None;
        };
        let origin = (node, idx);
        let creator_parent = Some((node, 2));
        let mut counters = self.inherit(parent, sibling);
        Self::apply(style, &mut counters, origin, creator_parent);
        let mut text = String::new();
        let mut has_counter = false;
        for item in &data.items[..data.alt_start] {
            match item {
                ContentItem::String(value) => text.push_str(value),
                ContentItem::Counter(name, style) => {
                    let index =
                        Self::ensure(&mut counters, name.0.as_ref(), origin, creator_parent);
                    text.push_str(&format_counter(counters[index].value, style));
                    has_counter = true;
                }
                ContentItem::Counters(name, separator, style) => {
                    Self::ensure(&mut counters, name.0.as_ref(), origin, creator_parent);
                    let values: Vec<_> = counters
                        .iter()
                        .filter(|c| c.name == name.0.as_ref())
                        .map(|c| format_counter(c.value, style))
                        .collect();
                    text.push_str(&values.join(separator));
                    has_counter = true;
                }
                _ => {}
            }
        }
        if has_counter {
            self.text.insert(origin, text);
        }
        self.previous = counters.clone();
        Some(counters)
    }

    fn element(
        &mut self,
        doc: &BaseDocument,
        node: NodeId,
        parent: &[Counter],
        sibling: &[Counter],
    ) -> Option<Vec<Counter>> {
        let style = doc.nodes[node].primary_styles()?;
        let display = style.clone_display();
        if display.is_none() {
            return None;
        }
        let mut counters = self.inherit(parent, sibling);
        if !display.is_contents() {
            Self::apply(
                &style,
                &mut counters,
                (node, 2),
                doc.nodes[node].parent.map(|id| (id, 2)),
            );
        }
        self.previous = counters.clone();
        let mut previous_sibling = self
            .pseudo(doc, node, 1, &counters, &[])
            .unwrap_or_default();
        for child in doc.nodes[node].children.iter().copied() {
            if let Some(values) = self.element(doc, child, &counters, &previous_sibling) {
                previous_sibling = values;
            }
        }
        self.pseudo(doc, node, 0, &counters, &previous_sibling);
        Some(counters)
    }
}

pub(crate) fn resolve_generated_counters(doc: &mut BaseDocument) {
    let mut resolver = Resolver::default();
    resolver.element(doc, doc.root_element().id, &[], &[]);
    for (key, text) in &resolver.text {
        if doc.generated_counter_text.get(key) != Some(text) {
            doc.nodes[key.0].insert_damage(ALL_DAMAGE);
        }
    }
    for key in doc.generated_counter_text.keys() {
        if !resolver.text.contains_key(key) {
            if let Some(node) = doc.nodes.get_mut(key.0) {
                node.insert_damage(ALL_DAMAGE);
            }
        }
    }
    doc.generated_counter_text = resolver.text;
}

/// Common predefined styles. Unknown named styles use the CSS decimal fallback.
pub(super) fn format_counter(value: i32, style: &CounterStyle) -> String {
    let name = match style {
        CounterStyle::None => return String::new(),
        CounterStyle::String(value) => return value.to_string(),
        CounterStyle::Name(name) => name.0.as_ref(),
        _ => return value.to_string(),
    };
    match name {
        "decimal-leading-zero" => {
            if value < 0 {
                format!("-{:02}", value.unsigned_abs())
            } else {
                format!("{value:02}")
            }
        }
        "lower-alpha" | "lower-latin" | "upper-alpha" | "upper-latin" if value > 0 => {
            let mut number = value as u32;
            let mut digits = Vec::new();
            while number > 0 {
                number -= 1;
                digits.push(b'a' + (number % 26) as u8);
                number /= 26;
            }
            let mut text: String = digits.into_iter().rev().map(char::from).collect();
            if name.starts_with("upper") {
                text.make_ascii_uppercase();
            }
            text
        }
        "lower-roman" | "upper-roman" if (1..=3999).contains(&value) => {
            let mut number = value;
            let mut text = String::new();
            for (amount, letters) in [
                (1000, "M"),
                (900, "CM"),
                (500, "D"),
                (400, "CD"),
                (100, "C"),
                (90, "XC"),
                (50, "L"),
                (40, "XL"),
                (10, "X"),
                (9, "IX"),
                (5, "V"),
                (4, "IV"),
                (1, "I"),
            ] {
                while number >= amount {
                    text.push_str(letters);
                    number -= amount;
                }
            }
            if name.starts_with("lower") {
                text.make_ascii_lowercase();
            }
            text
        }
        _ => value.to_string(),
    }
}
