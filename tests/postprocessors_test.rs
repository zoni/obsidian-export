use obsidian_export::{Context, Exporter, MarkdownEvents, PostprocessorResult};
use pretty_assertions::assert_eq;
use pulldown_cmark::{CowStr, Event};
use serde_yaml::Value;
use std::fs::{read_to_string, remove_file};
use std::path::PathBuf;
use tempfile::TempDir;

/// This postprocessor replaces any instance of "foo" with "bar" in the note body.
fn foo_to_bar(
    ctx: Context,
    events: MarkdownEvents,
) -> (Context, MarkdownEvents, PostprocessorResult) {
    let events = events
        .into_iter()
        .map(|event| match event {
            Event::Text(text) => Event::Text(CowStr::from(text.replace("foo", "bar"))),
            event => event,
        })
        .collect();
    (ctx, events, PostprocessorResult::Continue)
}

/// This postprocessor appends "bar: baz" to frontmatter.
fn append_frontmatter(
    mut ctx: Context,
    events: MarkdownEvents,
) -> (Context, MarkdownEvents, PostprocessorResult) {
    ctx.frontmatter.insert(
        Value::String("bar".to_string()),
        Value::String("baz".to_string()),
    );
    (ctx, events, PostprocessorResult::Continue)
}

// The purpose of this test to verify the `append_frontmatter` postprocessor is called to extend
// the frontmatter, and the `foo_to_bar` postprocessor is called to replace instances of "foo" with
// "bar" (only in the note body).
#[test]
fn test_postprocessors() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.add_postprocessor(&foo_to_bar);
    exporter.add_postprocessor(&append_frontmatter);

    exporter.run().unwrap();

    let expected = read_to_string("tests/testdata/expected/postprocessors/Note.md").unwrap();
    let actual = read_to_string(tmp_dir.path().clone().join(PathBuf::from("Note.md"))).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_postprocessor_stophere() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors"),
        tmp_dir.path().to_path_buf(),
    );

    exporter.add_postprocessor(&|ctx, mdevents| (ctx, mdevents, PostprocessorResult::StopHere));
    exporter.add_postprocessor(&|_, _| panic!("should not be called due to above processor"));
    exporter.run().unwrap();
}

#[test]
fn test_postprocessor_stop_and_skip() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let note_path = tmp_dir.path().clone().join(PathBuf::from("Note.md"));

    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.run().unwrap();

    assert!(note_path.exists());
    remove_file(&note_path).unwrap();

    exporter
        .add_postprocessor(&|ctx, mdevents| (ctx, mdevents, PostprocessorResult::StopAndSkipNote));
    exporter.run().unwrap();

    assert!(!note_path.exists());
}

#[test]
fn test_postprocessor_change_destination() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let original_note_path = tmp_dir.path().clone().join(PathBuf::from("Note.md"));
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.run().unwrap();

    assert!(original_note_path.exists());
    remove_file(&original_note_path).unwrap();

    exporter.add_postprocessor(&|mut ctx, mdevents| {
        ctx.destination.set_file_name("MovedNote.md");
        (ctx, mdevents, PostprocessorResult::Continue)
    });
    exporter.run().unwrap();

    let new_note_path = tmp_dir.path().clone().join(PathBuf::from("MovedNote.md"));
    assert!(!original_note_path.exists());
    assert!(new_note_path.exists());
}
