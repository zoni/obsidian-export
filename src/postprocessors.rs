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

pub fn filter_by_tags(
    skip_tags: Vec<String>,
    only_tags: Vec<String>,
) -> impl Fn(&mut Context, &mut MarkdownEvents) -> PostprocessorResult {
    move |context: &mut Context, _events: &mut MarkdownEvents| -> PostprocessorResult {
        if !skip_tags.is_empty() || !only_tags.is_empty() {
            match context.frontmatter.get("tags") {
                Some(Value::Sequence(tags)) => {
                    let skip = skip_tags
                        .iter()
                        .any(|tag| tags.contains(&Value::String(tag.to_string())));
                    let include = only_tags.is_empty()
                        || only_tags
                            .iter()
                            .any(|tag| tags.contains(&Value::String(tag.to_string())));
                    if skip || !include {
                        PostprocessorResult::StopAndSkipNote
                    } else {
                        PostprocessorResult::Continue
                    }
                }
                _ => PostprocessorResult::Continue,
            }
        } else {
            PostprocessorResult::Continue
        }
    }
}
