//! A collection of officially maintained [postprocessors][crate::Postprocessor].

use super::{Context, MarkdownEvents, PostprocessorResult};
use pulldown_cmark::Event;
use serde_yaml::Value;

/// This postprocessor converts all soft line breaks to hard line breaks. Enabling this mimics
/// Obsidian's _'Strict line breaks'_ setting.
pub fn softbreaks_to_hardbreaks(
    _context: &mut Context,
    events: &mut MarkdownEvents,
) -> PostprocessorResult {
    for event in events.iter_mut() {
        if event == &Event::SoftBreak {
            *event = Event::HardBreak;
        }
    }
    PostprocessorResult::Continue
}

/// Create a new postprocessor which skips any notes that don't have the given `yaml_filter_key`
/// set to a truthy value in their frontmatter.
///
/// Initialize this as follows:
///
/// ```
/// use obsidian_export::Exporter;
/// use obsidian_export::postprocessors::create_frontmatter_filter;
/// # use std::path::PathBuf;
/// # use tempfile::TempDir;
///
/// # let tmp_dir = TempDir::new().expect("failed to make tempdir");
/// # let source = PathBuf::from("tests/testdata/input/postprocessors");
/// # let destination = tmp_dir.path().to_path_buf();
/// let mut exporter = Exporter::new(source, destination);
/// let filter = create_frontmatter_filter("export");
/// exporter.add_postprocessor(&filter);
/// # exporter.run().unwrap();
/// ```
pub fn create_frontmatter_filter(
    yaml_filter_key: &str,
) -> impl Fn(&mut Context, &mut MarkdownEvents) -> PostprocessorResult {
    let key = serde_yaml::Value::String(yaml_filter_key.to_string());

    move |context: &mut Context, _events: &mut MarkdownEvents| {
        match context.frontmatter.get(&key) {
            Some(Value::Bool(true)) => PostprocessorResult::Continue,
            _ => PostprocessorResult::StopAndSkipNote,
        }
    }
}
