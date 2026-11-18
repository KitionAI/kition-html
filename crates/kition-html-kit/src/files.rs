//! The parameters of this kit's commands that name files or folders on disk (KHP A28), declared
//! as typed files in the commands' schemas (`kition_kit::files`). The HTML kit edits its source
//! in memory: no command names a file.

use kition_kit::files::FileParam;

/// Every file or folder parameter of the kit's commands.
pub const FILES: &[FileParam] = &[];

#[cfg(test)]
mod tests {
    use kition_kit::KitDocument;

    use super::*;

    /// Parameters whose names read like files but are not.
    const NOT_FILES: &[(&str, &str)] = &[];

    #[test]
    fn every_file_parameter_is_declared() {
        let doc = crate::HtmlDocument::create("Untitled.html", 64, 64)
            .unwrap_or_else(|error| panic!("{error}"));
        let problems = kition_kit::files::check(&doc.commands(), FILES, NOT_FILES);
        assert!(problems.is_empty(), "{problems:#?}");
    }
}
