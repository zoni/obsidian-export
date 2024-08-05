//! A collection of officially maintained [postprocessors][crate::Postprocessor].

use super::{Context, MarkdownEvents, PostprocessorResult};
use pulldown_cmark::{CowStr, Event, Tag};
use regex::Regex;

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

/// This postprocessor removes all Obsidian comments from a file excluding codeblocks. Enabling this
/// prohibits comments from being exported but leaves them untouched in the original files
pub fn remove_obsidian_comments(
    _context: &mut Context,
    events: &mut MarkdownEvents,
) -> PostprocessorResult {
    let mut output = Vec::with_capacity(events.len());
    let mut inside_comment = false;
    let mut inside_codeblock = false;

    for event in &mut *events {
        output.push(event.to_owned());

        match event {
            Event::Text(ref text) => {
                if !text.contains("%%") {
                    if inside_comment {
                        output.pop(); //Inside block comment so remove
                    }
                    continue;
                } else if inside_codeblock {
                    continue; //Skip anything inside codeblocks
                }

                output.pop();

                if inside_comment {
                    inside_comment = false;
                    continue;
                }

                if !text.eq(&CowStr::from("%%")) {
                    let re = Regex::new(r"%%.*?%%").unwrap();
                    let result = re.replace_all(text, "").to_string();
                    output.push(Event::Text(CowStr::from(result)));
                    continue;
                }

                inside_comment = true;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                inside_codeblock = true;
            }
            Event::End(Tag::CodeBlock(_)) => {
                inside_codeblock = false;
            }
            Event::End(Tag::Paragraph) => {
                if output[output.len() - 2] == Event::Start(Tag::Paragraph) {
                    // If the comment was the only item on the line remove the start and end paragraph events to remove the \n in the output file.
                    output.pop();
                    output.pop();
                }
            }
            _ => {
                if inside_comment {
                    output.pop();
                }
            }
        }
    }
    *events = output;
    PostprocessorResult::Continue
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
