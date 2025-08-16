//! A collection of officially maintained [postprocessors][crate::Postprocessor].

use pulldown_cmark::Event;
use serde_yaml::Value;

use super::{Context, MarkdownEvents, PostprocessorResult};

/// This postprocessor converts all soft line breaks to hard line breaks. Enabling this mimics
/// Obsidian's _'Strict line breaks'_ setting.
pub fn softbreaks_to_hardbreaks(
    _context: &mut Context,
    events: &mut MarkdownEvents<'_>,
) -> PostprocessorResult {
    for event in events.iter_mut() {
        if event == &Event::SoftBreak {
            *event = Event::HardBreak;
        }
    }
    PostprocessorResult::Continue
}

/// In addition to HTML comments "<!-- my comment -->", Obsidian syntax includes a second comment
/// syntax "%% my comment %%". This function converts Obsidian-style comments to the standard HTML
/// style.
///
/// Because this step is done when the document is already parsed from markdown, any embedded
/// markup inside comments will be elided from the output.
pub fn parse_obsidian_comments(
    context: &mut Context,
    events: &mut MarkdownEvents<'_>,
) -> PostprocessorResult {
    let mut in_comment = false;
    let mut comment_acc = String::new();
    let input = std::mem::take(events);
    let mut output = MarkdownEvents::new();

    for event in input {
        match event {
            Event::Text(s) => {
                for (idx, text) in s.split("%%").enumerate() {
                    if idx > 0 {
                        if in_comment && !comment_acc.is_empty() {
                            output.push(Event::InlineHtml(format!("<!--{comment_acc}-->").into()));
                            comment_acc.clear();
                        }

                        in_comment = !in_comment;
                    }

                    if !text.is_empty() {
                        if in_comment {
                            comment_acc.push_str(text);
                        } else {
                            output.push(Event::Text(text.to_owned().into()));
                        }
                    }
                }
            }
            _ => {
                if !in_comment {
                    output.push(event);
                }
            }
        }
    }

    assert!(
        !in_comment,
        "Unmatched comment delimiter in {}",
        context.destination.display()
    );

    std::mem::swap(events, &mut output);

    PostprocessorResult::Continue
}

pub fn filter_by_tags(
    skip_tags: Vec<String>,
    only_tags: Vec<String>,
) -> impl Fn(&mut Context, &mut MarkdownEvents<'_>) -> PostprocessorResult {
    move |context: &mut Context, _events: &mut MarkdownEvents<'_>| -> PostprocessorResult {
        match context.frontmatter.get("tags") {
            None => filter_by_tags_(&[], &skip_tags, &only_tags),
            Some(Value::Sequence(tags)) => filter_by_tags_(tags, &skip_tags, &only_tags),
            _ => PostprocessorResult::Continue,
        }
    }
}

fn filter_by_tags_(
    tags: &[Value],
    skip_tags: &[String],
    only_tags: &[String],
) -> PostprocessorResult {
    let skip = skip_tags
        .iter()
        .any(|tag| tags.contains(&Value::String(tag.clone())));
    let include = only_tags.is_empty()
        || only_tags
            .iter()
            .any(|tag| tags.contains(&Value::String(tag.clone())));

    if skip || !include {
        PostprocessorResult::StopAndSkipNote
    } else {
        PostprocessorResult::Continue
    }
}

#[test]
fn test_filter_tags() {
    let tags = vec![
        Value::String("skip".into()),
        Value::String("publish".into()),
    ];
    let empty_tags = vec![];
    assert_eq!(
        filter_by_tags_(&empty_tags, &[], &[]),
        PostprocessorResult::Continue,
        "When no exclusion & inclusion are specified, files without tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &[]),
        PostprocessorResult::Continue,
        "When no exclusion & inclusion are specified, files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["exclude".into()], &[]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&empty_tags, &["exclude".into()], &[]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files without tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &["publish".into()]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&empty_tags, &[], &["include".into()]),
        PostprocessorResult::StopAndSkipNote,
        "When inclusion tags are specified files without tags are excluded"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &["include".into()]),
        PostprocessorResult::StopAndSkipNote,
        "When exclusion tags don't match files with tags are exluded"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["skip".into()], &["skip".into()]),
        PostprocessorResult::StopAndSkipNote,
        "When both inclusion and exclusion tags are the same exclusion wins"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["skip".into()], &["publish".into()]),
        PostprocessorResult::StopAndSkipNote,
        "When both inclusion and exclusion tags match exclusion wins"
    );
}
