//! A collection of officially maintained [postprocessors][crate::Postprocessor].

use super::{Context, MarkdownEvents, PostprocessorResult};
use pulldown_cmark::Event;

/// This postprocessor converts all soft line breaks to hard line breaks. Enabling this mimics
/// Obsidian's _'Strict line breaks'_ setting.
pub fn softbreaks_to_hardbreaks(
    context: Context,
    events: MarkdownEvents,
) -> (Context, MarkdownEvents, PostprocessorResult) {
    let events = events
        .into_iter()
        .map(|event| match event {
            Event::SoftBreak => Event::HardBreak,
            _ => event,
        })
        .collect();
    (context, events, PostprocessorResult::Continue)
}
