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

/// This postprocessor converts returns a new function (closure) that searches for the specified 
/// yaml_filter_key in a notes frontmatter. If it does not find a yaml_filter_key: true in the YAML, 
/// it tells the exporter to StopandSkipNote
pub fn create_yaml_includer(
    yaml_filter_key: &str,
) -> impl Fn(&mut Context, &mut MarkdownEvents) -> PostprocessorResult {
    let key = serde_yaml::Value::String(yaml_filter_key.to_string());

    // This bit creates and returns the closure. The `move` statement is needed to make it take
    // ownership of `key` above.
    move |context: &mut Context, _events: &mut MarkdownEvents| {
        match context.frontmatter.get(&key) {
            Some(Value::Bool(true)) => PostprocessorResult::Continue,
            _ => PostprocessorResult::StopAndSkipNote,
        }
    }
}
