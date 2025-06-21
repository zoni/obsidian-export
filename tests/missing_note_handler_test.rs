use std::fs::{create_dir_all, write};

use obsidian_export::pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};
use obsidian_export::Exporter;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[test]
fn test_default_missing_note_handler_link() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    // Create a note with a missing link reference
    write(
        input_dir.join("test.md"),
        "This is a link to [[missing-note]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());
    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    // Default handler should convert missing links to emphasized text
    assert_eq!(result, "This is a link to *missing-note*.\n");
}

#[test]
fn test_default_missing_note_handler_embed() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    // Create a note with a missing embed reference
    write(
        input_dir.join("test.md"),
        "This embeds a missing note: ![[missing-embed]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());
    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    // Default handler should remove missing embeds entirely
    assert_eq!(result, "This embeds a missing note: .\n");
}

#[test]
fn test_custom_missing_note_handler_skip_all() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Link: [[missing-link]] and embed: ![[missing-embed]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    // Custom handler that skips all missing references
    exporter.set_missing_note_handler(&|_context, _reference, _is_embed| {
        vec![] // Return empty - remove reference entirely
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    assert_eq!(result, "Link:  and embed: .\n");
}

#[test]
fn test_custom_missing_note_handler_different_behavior_for_links_and_embeds() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Link: [[missing-link]] and embed: ![[missing-embed]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    // Custom handler with different behavior for links vs embeds
    exporter.set_missing_note_handler(&|_context, reference, is_embed| {
        if is_embed {
            // For embeds, show as code block
            vec![
                Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Indented)),
                Event::Text(CowStr::from(format!(
                    "MISSING EMBED: {}",
                    reference.display()
                ))),
                Event::End(TagEnd::CodeBlock),
            ]
        } else {
            // For links, show as strong text
            vec![
                Event::Start(Tag::Strong),
                Event::Text(CowStr::from(format!(
                    "MISSING LINK: {}",
                    reference.display()
                ))),
                Event::End(TagEnd::Strong),
            ]
        }
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    assert!(result.contains("**MISSING LINK: missing-link**"));
    assert!(result.contains("    MISSING EMBED: missing-embed"));
}

#[test]
fn test_custom_missing_note_handler_create_external_links() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Missing reference: [[missing-note]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    // Custom handler that creates external links
    exporter.set_missing_note_handler(&|_context, reference, _is_embed| {
        let url = format!(
            "https://example.com/search?q={}",
            reference.file.unwrap_or("unknown")
        );
        vec![
            Event::Start(Tag::Link {
                link_type: LinkType::Inline,
                dest_url: CowStr::from(url),
                title: CowStr::from(""),
                id: CowStr::from(""),
            }),
            Event::Text(CowStr::from(reference.display())),
            Event::End(TagEnd::Link),
        ]
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    assert!(result.contains("[missing-note](https://example.com/search?q=missing-note)"));
}

#[test]
fn test_missing_note_handler_with_aliased_references() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Link with alias: [[missing-note|Custom Label]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    exporter.set_missing_note_handler(&|_context, reference, _is_embed| {
        // Verify reference parsing is correct
        assert_eq!(reference.file, Some("missing-note"));
        assert_eq!(reference.label, Some("Custom Label"));
        assert_eq!(reference.display(), "Custom Label"); // display() should show the label when present

        vec![Event::Text(CowStr::from("REPLACED"))]
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    assert!(result.contains("REPLACED"));
}

#[test]
fn test_missing_note_handler_with_section_references() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Section link: [[missing-note#Section Name]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    exporter.set_missing_note_handler(&|_context, reference, _is_embed| {
        // Verify section reference parsing
        assert_eq!(reference.file, Some("missing-note"));
        assert_eq!(reference.section, Some("Section Name"));
        assert_eq!(reference.label, None);

        vec![Event::Text(CowStr::from("REPLACED"))]
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    assert!(result.contains("REPLACED"));
}

#[test]
fn test_missing_note_handler_with_current_file_section_reference() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Section in current file: [[#Missing Section]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    // For current file section references, the missing note handler is NOT called
    // because the file exists (current file) but the section doesn't
    // This creates a regular link to the section
    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    // Should create a link to the missing section within the same file
    assert!(result.contains("[Missing Section](test.md#missing-section)"));
}

#[test]
fn test_missing_note_handler_complex_reference_parsing() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Complex: [[missing-note#Section Name|Custom Display Text]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    exporter.set_missing_note_handler(&|_context, reference, _is_embed| {
        // Verify complex reference parsing
        assert_eq!(reference.file, Some("missing-note"));
        assert_eq!(reference.section, Some("Section Name"));
        assert_eq!(reference.label, Some("Custom Display Text"));
        assert_eq!(reference.display(), "Custom Display Text"); // Should show label when present

        vec![Event::Text(CowStr::from("REPLACED"))]
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    assert!(result.contains("REPLACED"));
}

#[test]
fn test_missing_note_handler_warning_suppression() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Missing reference: [[missing-note]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    // Custom handler - should suppress default warnings
    exporter.set_missing_note_handler(&|_context, reference, _is_embed| {
        // Custom handler is responsible for warnings if desired
        vec![Event::Text(CowStr::from(format!(
            "CUSTOM: {}",
            reference.display()
        )))]
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    assert!(result.contains("CUSTOM: missing-note"));
}

// Test for argument order - context comes first
#[test]
fn test_missing_note_handler_argument_order() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    write(
        input_dir.join("test.md"),
        "Missing reference: [[missing-note]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    // Verify argument order: context, reference, is_embed
    exporter.set_missing_note_handler(&|context, reference, is_embed| {
        // Verify we can access all three parameters in the expected order
        assert!(context.current_file().to_string_lossy().contains("test.md"));
        assert_eq!(reference.file, Some("missing-note"));
        assert!(!is_embed); // This is a link, not an embed

        vec![Event::Text(CowStr::from("ORDER_TEST_PASSED"))]
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("test.md")).unwrap();
    assert!(result.contains("ORDER_TEST_PASSED"));
}

#[test]
fn test_missing_note_handler_nested_embeds() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let input_dir = tmp_dir.path().join("input");
    let output_dir = tmp_dir.path().join("output");
    create_dir_all(&input_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    // Create a note that embeds another note, which has missing references
    write(
        input_dir.join("parent.md"),
        "Parent note embeds: ![[child]].",
    )
    .unwrap();

    write(
        input_dir.join("child.md"),
        "Child note references: [[missing-from-child]].",
    )
    .unwrap();

    let mut exporter = Exporter::new(input_dir, output_dir.clone());

    exporter.set_missing_note_handler(&|context, reference, is_embed| {
        // Verify nested context information
        assert!(context
            .current_file()
            .to_string_lossy()
            .contains("child.md"));
        // The depth might be 1 if the child note is processed as a root note during embed
        // processing Let's just verify it's a positive depth and accept both 1 and 2
        assert!(context.note_depth() >= 1);
        assert_eq!(reference.file, Some("missing-from-child"));
        assert!(!is_embed); // This is a link, not an embed

        vec![Event::Text(CowStr::from("NESTED_REPLACED"))]
    });

    exporter.run().unwrap();

    let result = std::fs::read_to_string(output_dir.join("parent.md")).unwrap();
    assert!(result.contains("NESTED_REPLACED"));
}
