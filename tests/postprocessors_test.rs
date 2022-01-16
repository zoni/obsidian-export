use obsidian_export::postprocessors::{create_frontmatter_filter, softbreaks_to_hardbreaks};
use obsidian_export::{Context, Exporter, MarkdownEvents, PostprocessorResult};
use pretty_assertions::assert_eq;
use pulldown_cmark::{CowStr, Event};
use serde_yaml::Value;
use std::fs::{read_to_string, remove_file};
use std::path::PathBuf;
use tempfile::TempDir;

/// This postprocessor replaces any instance of "foo" with "bar" in the note body.
fn foo_to_bar(_ctx: &mut Context, events: &mut MarkdownEvents) -> PostprocessorResult {
    for event in events.iter_mut() {
        if let Event::Text(text) = event {
            *event = Event::Text(CowStr::from(text.replace("foo", "bar")))
        }
    }
    PostprocessorResult::Continue
}

/// This postprocessor appends "bar: baz" to frontmatter.
fn append_frontmatter(ctx: &mut Context, _events: &mut MarkdownEvents) -> PostprocessorResult {
    ctx.frontmatter.insert(
        Value::String("bar".to_string()),
        Value::String("baz".to_string()),
    );
    PostprocessorResult::Continue
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

    exporter.add_postprocessor(&|_ctx, _mdevents| PostprocessorResult::StopHere);
    exporter.add_embed_postprocessor(&|_ctx, _mdevents| PostprocessorResult::StopHere);
    exporter.add_postprocessor(&|_, _| panic!("should not be called due to above processor"));
    exporter.add_embed_postprocessor(&|_, _| panic!("should not be called due to above processor"));
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

    exporter.add_postprocessor(&|_ctx, _mdevents| PostprocessorResult::StopAndSkipNote);
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

    exporter.add_postprocessor(&|ctx, _mdevents| {
        ctx.destination.set_file_name("MovedNote.md");
        PostprocessorResult::Continue
    });
    exporter.run().unwrap();

    let new_note_path = tmp_dir.path().clone().join(PathBuf::from("MovedNote.md"));
    assert!(!original_note_path.exists());
    assert!(new_note_path.exists());
}

// The purpose of this test to verify the `append_frontmatter` postprocessor is called to extend
// the frontmatter, and the `foo_to_bar` postprocessor is called to replace instances of "foo" with
// "bar" (only in the note body).
#[test]
fn test_embed_postprocessors() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.add_embed_postprocessor(&foo_to_bar);
    // Should have no effect with embeds:
    exporter.add_embed_postprocessor(&append_frontmatter);

    exporter.run().unwrap();

    let expected =
        read_to_string("tests/testdata/expected/postprocessors/Note_embed_postprocess_only.md")
            .unwrap();
    let actual = read_to_string(tmp_dir.path().clone().join(PathBuf::from("Note.md"))).unwrap();
    assert_eq!(expected, actual);
}

// When StopAndSkipNote is used with an embed_preprocessor, it should skip the embedded note but
// continue with the rest of the note.
#[test]
fn test_embed_postprocessors_stop_and_skip() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.add_embed_postprocessor(&|_ctx, _mdevents| PostprocessorResult::StopAndSkipNote);

    exporter.run().unwrap();

    let expected =
        read_to_string("tests/testdata/expected/postprocessors/Note_embed_stop_and_skip.md")
            .unwrap();
    let actual = read_to_string(tmp_dir.path().clone().join(PathBuf::from("Note.md"))).unwrap();
    assert_eq!(expected, actual);
}

// This test verifies that the context which is passed to an embed postprocessor is actually
// correct. Primarily, this means the frontmatter should reflect that of the note being embedded as
// opposed to the frontmatter of the root note.
#[test]
fn test_embed_postprocessors_context() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors"),
        tmp_dir.path().to_path_buf(),
    );

    exporter.add_postprocessor(&|ctx, _mdevents| {
        if ctx.current_file() != &PathBuf::from("Note.md") {
            return PostprocessorResult::Continue;
        }
        let is_root_note = ctx
            .frontmatter
            .get(&Value::String("is_root_note".to_string()))
            .unwrap_or(&Value::Bool(false));
        if is_root_note != &Value::Bool(true) {
            // NOTE: Test failure may not give output consistently because the test binary affects
            // how output is captured and printed in the thread running this postprocessor. Just
            // run the test a couple times until the error shows up.
            panic!(
                "postprocessor: expected is_root_note in {} to be true, got false",
                &ctx.current_file().display()
            )
        }
        PostprocessorResult::Continue
    });
    exporter.add_embed_postprocessor(&|ctx, _mdevents| {
        let is_root_note = ctx
            .frontmatter
            .get(&Value::String("is_root_note".to_string()))
            .unwrap_or(&Value::Bool(false));
        if is_root_note == &Value::Bool(true) {
            // NOTE: Test failure may not give output consistently because the test binary affects
            // how output is captured and printed in the thread running this postprocessor. Just
            // run the test a couple times until the error shows up.
            panic!(
                "embed_postprocessor: expected is_root_note in {} to be false, got true",
                &ctx.current_file().display()
            )
        }
        PostprocessorResult::Continue
    });

    exporter.run().unwrap();
}

#[test]
fn test_softbreaks_to_hardbreaks() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.add_postprocessor(&softbreaks_to_hardbreaks);
    exporter.run().unwrap();

    let expected =
        read_to_string("tests/testdata/expected/postprocessors/hard_linebreaks.md").unwrap();
    let actual = read_to_string(
        tmp_dir
            .path()
            .clone()
            .join(PathBuf::from("hard_linebreaks.md")),
    )
    .unwrap();
    assert_eq!(expected, actual);
}

// This test verifies that yaml inclusion works as desired
// ONLY when a .md file has the specified key set to a YAML true should it work
#[test]
fn test_yaml_inclusion() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let files = ["exlude_me.md", "include_me.md", "no_yaml.md"];
    let desired = [false, true, false];

    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors/yaml-filtering"),
        tmp_dir.path().to_path_buf(),
    );
    let yaml_postprocessor = create_frontmatter_filter("export");
    exporter.add_postprocessor(&yaml_postprocessor);

    // Run the exporter
    exporter.run().unwrap();

    // Check that each file is included or excluded correctly
    files
        .iter()
        .zip(desired.iter())
        .map(|(f, b)| {
            let note_path = tmp_dir.path().clone().join(PathBuf::from(f));
            assert!(note_path.exists() == *b);
        })
        .collect()
}

// This test verifies that yaml inclusion works as desired for the embedded post-processor
// The behavior should be that the file is only embedded *if* it's frontmatter contains export: true
#[test]
fn test_yaml_inclusion_embedded() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/postprocessors/yaml-filtering"),
        tmp_dir.path().to_path_buf(),
    );

    let yaml_postprocessor = create_frontmatter_filter("export");
    exporter.add_postprocessor(&yaml_postprocessor);
    exporter.add_embed_postprocessor(&yaml_postprocessor);

    // Run the exporter
    exporter.run().unwrap();

    let expected =
        read_to_string("tests/testdata/expected/postprocessors/yaml-filtering/included_embed.md")
            .unwrap();
    let actual =
        read_to_string(tmp_dir.path().clone().join(PathBuf::from("include_me.md"))).unwrap();

    assert_eq!(expected, actual);
}
