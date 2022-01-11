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

// This function takes as input the YAML key to look for, then returns a new function (technically:
// a closure) which matches the signature of a postprocessor.
//
// This use of dynamic function building allows the capturing of the configuration (in this case
// the YAML key) without needing to store this data within the Exporter struct.
//
// (Ideally we could mark the return value as `-> impl Postprocessor` for readability, but we
// cannot use a type alias here, which is what `Postprocessor` is)
pub fn create_yaml_includer(
    yaml_inclusion_key: &str,
) -> impl Fn(&mut Context, &mut MarkdownEvents) -> PostprocessorResult {
    let key = serde_yaml::Value::String(yaml_inclusion_key.to_string());

    // This bit creates and returns the closure. The `move` statement is needed to make it take
    // ownership of `key` above.
    move |context: &mut Context, _events: &mut MarkdownEvents| {
        match context.frontmatter.get(&key) {
            Some(Value::Bool(true)) => PostprocessorResult::Continue,
            _ => PostprocessorResult::StopAndSkipNote,
        }
    }
}
