use obsidian_export::{ExportError, Exporter, FrontmatterStrategy};
use pretty_assertions::assert_eq;
use std::fs::{create_dir, read_to_string, set_permissions, File, Permissions};
use std::io::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;
use walkdir::WalkDir;

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_main_variants_with_default_options() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    Exporter::new(
        PathBuf::from("tests/testdata/input/main-samples/"),
        tmp_dir.path().to_path_buf(),
    )
    .run()
    .expect("exporter returned error");

    let walker = WalkDir::new("tests/testdata/expected/main-samples/")
        // Without sorting here, different test runs may trigger the first assertion failure in
        // unpredictable order.
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter();
    for entry in walker {
        let entry = entry.unwrap();
        if entry.metadata().unwrap().is_dir() {
            continue;
        };
        let filename = entry.file_name().to_string_lossy().into_owned();
        let expected = read_to_string(entry.path()).unwrap_or_else(|_| {
            panic!(
                "failed to read {} from testdata/expected/main-samples/",
                entry.path().display()
            )
        });
        let actual = read_to_string(tmp_dir.path().join(PathBuf::from(&filename)))
            .unwrap_or_else(|_| panic!("failed to read {} from temporary exportdir", filename));

        assert_eq!(
            expected, actual,
            "{} does not have expected content",
            filename
        );
    }
}

#[test]
fn test_frontmatter_never() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/main-samples/"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.frontmatter_strategy(FrontmatterStrategy::Never);
    exporter.run().expect("exporter returned error");

    let expected = "Note with frontmatter.\n";
    let actual = read_to_string(
        tmp_dir
            .path()
            .join(PathBuf::from("note-with-frontmatter.md")),
    )
    .unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn test_frontmatter_always() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/main-samples/"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.frontmatter_strategy(FrontmatterStrategy::Always);
    exporter.run().expect("exporter returned error");

    // Note without frontmatter should have empty frontmatter added.
    let expected = "---\n---\n\nNote without frontmatter.\n";
    let actual = read_to_string(
        tmp_dir
            .path()
            .join(PathBuf::from("note-without-frontmatter.md")),
    )
    .unwrap();
    assert_eq!(expected, actual);

    // Note with frontmatter should remain untouched.
    let expected = "---\nFoo: bar\n---\n\nNote with frontmatter.\n";
    let actual = read_to_string(
        tmp_dir
            .path()
            .join(PathBuf::from("note-with-frontmatter.md")),
    )
    .unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_exclude() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    Exporter::new(
        PathBuf::from("tests/testdata/input/main-samples/"),
        tmp_dir.path().to_path_buf(),
    )
    .run()
    .expect("exporter returned error");

    let excluded_note = tmp_dir.path().join(PathBuf::from("excluded-note.md"));
    assert!(
        !excluded_note.exists(),
        "exluded-note.md was found in tmpdir, but should be absent due to .export-ignore rules"
    );
}

#[test]
fn test_single_file_to_dir() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    Exporter::new(
        PathBuf::from("tests/testdata/input/single-file/note.md"),
        tmp_dir.path().to_path_buf(),
    )
    .run()
    .unwrap();

    assert_eq!(
        read_to_string("tests/testdata/expected/single-file/note.md").unwrap(),
        read_to_string(tmp_dir.path().join(PathBuf::from("note.md"))).unwrap(),
    );
}

#[test]
fn test_single_file_to_file() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let dest = tmp_dir.path().join(PathBuf::from("export.md"));

    Exporter::new(
        PathBuf::from("tests/testdata/input/single-file/note.md"),
        dest.clone(),
    )
    .run()
    .unwrap();

    assert_eq!(
        read_to_string("tests/testdata/expected/single-file/note.md").unwrap(),
        read_to_string(&dest).unwrap(),
    );
}

#[test]
fn test_start_at_subdir() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/start-at/"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.start_at(PathBuf::from("tests/testdata/input/start-at/subdir"));
    exporter.run().unwrap();

    let expected = if cfg!(windows) {
        read_to_string("tests/testdata/expected/start-at/subdir/Note B.md")
            .unwrap()
            .replace('/', "\\")
    } else {
        read_to_string("tests/testdata/expected/start-at/subdir/Note B.md").unwrap()
    };

    assert_eq!(
        expected,
        read_to_string(tmp_dir.path().join(PathBuf::from("Note B.md"))).unwrap(),
    );
}

#[test]
fn test_start_at_file_within_subdir_destination_is_dir() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/start-at/"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.start_at(PathBuf::from(
        "tests/testdata/input/start-at/subdir/Note B.md",
    ));
    exporter.run().unwrap();

    let expected = if cfg!(windows) {
        read_to_string("tests/testdata/expected/start-at/single-file/Note B.md")
            .unwrap()
            .replace('/', "\\")
    } else {
        read_to_string("tests/testdata/expected/start-at/single-file/Note B.md").unwrap()
    };

    assert_eq!(
        expected,
        read_to_string(tmp_dir.path().join(PathBuf::from("Note B.md"))).unwrap(),
    );
}

#[test]
fn test_start_at_file_within_subdir_destination_is_file() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let dest = tmp_dir.path().join(PathBuf::from("note.md"));
    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/start-at/"),
        dest.clone(),
    );
    exporter.start_at(PathBuf::from(
        "tests/testdata/input/start-at/subdir/Note B.md",
    ));
    exporter.run().unwrap();

    let expected = if cfg!(windows) {
        read_to_string("tests/testdata/expected/start-at/single-file/Note B.md")
            .unwrap()
            .replace('/', "\\")
    } else {
        read_to_string("tests/testdata/expected/start-at/single-file/Note B.md").unwrap()
    };
    assert_eq!(expected, read_to_string(dest).unwrap(),);
}

#[test]
fn test_not_existing_source() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    let err = Exporter::new(
        PathBuf::from("tests/testdata/no-such-file.md"),
        tmp_dir.path().to_path_buf(),
    )
    .run()
    .unwrap_err();

    match err {
        ExportError::PathDoesNotExist { path: _ } => {}
        _ => panic!("Wrong error variant: {:?}", err),
    }
}

#[test]
fn test_not_existing_destination_with_source_dir() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    let err = Exporter::new(
        PathBuf::from("tests/testdata/input/main-samples/"),
        tmp_dir.path().to_path_buf().join("does-not-exist"),
    )
    .run()
    .unwrap_err();

    match err {
        ExportError::PathDoesNotExist { path: _ } => {}
        _ => panic!("Wrong error variant: {:?}", err),
    }
}

#[test]
// This test ensures that when source is a file, but destination points to a regular file
// inside of a non-existent directory, an error is raised instead of that directory path being
// created (like `mkdir -p`)
fn test_not_existing_destination_with_source_file() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    let err = Exporter::new(
        PathBuf::from("tests/testdata/input/main-samples/obsidian-wikilinks.md"),
        tmp_dir.path().to_path_buf().join("subdir/does-not-exist"),
    )
    .run()
    .unwrap_err();

    match err {
        ExportError::PathDoesNotExist { path: _ } => {}
        _ => panic!("Wrong error variant: {:?}", err),
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_source_no_permissions() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let src = tmp_dir.path().to_path_buf().join("source.md");
    let dest = tmp_dir.path().to_path_buf().join("dest.md");

    let mut file = File::create(&src).unwrap();
    file.write_all("Foo".as_bytes()).unwrap();
    set_permissions(&src, Permissions::from_mode(0o000)).unwrap();

    match Exporter::new(src, dest).run().unwrap_err() {
        ExportError::FileExportError { path: _, source } => match *source {
            ExportError::ReadError { path: _, source: _ } => {}
            _ => panic!("Wrong error variant for source, got: {:?}", source),
        },
        err => panic!("Wrong error variant: {:?}", err),
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_dest_no_permissions() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    let src = tmp_dir.path().to_path_buf().join("source.md");
    let dest = tmp_dir.path().to_path_buf().join("dest");

    let mut file = File::create(&src).unwrap();
    file.write_all("Foo".as_bytes()).unwrap();

    create_dir(&dest).unwrap();
    set_permissions(&dest, Permissions::from_mode(0o555)).unwrap();

    match Exporter::new(src, dest).run().unwrap_err() {
        ExportError::FileExportError { path: _, source } => match *source {
            ExportError::WriteError { path: _, source: _ } => {}
            _ => panic!("Wrong error variant for source, got: {:?}", source),
        },
        err => panic!("Wrong error variant: {:?}", err),
    }
}

#[test]
fn test_infinite_recursion() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    let err = Exporter::new(
        PathBuf::from("tests/testdata/input/infinite-recursion/"),
        tmp_dir.path().to_path_buf(),
    )
    .run()
    .unwrap_err();

    match err {
        ExportError::FileExportError { path: _, source } => match *source {
            ExportError::RecursionLimitExceeded { .. } => {}
            _ => panic!("Wrong error variant for source, got: {:?}", source),
        },
        err => panic!("Wrong error variant: {:?}", err),
    }
}

#[test]
fn test_no_recursive_embeds() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    let mut exporter = Exporter::new(
        PathBuf::from("tests/testdata/input/infinite-recursion/"),
        tmp_dir.path().to_path_buf(),
    );
    exporter.process_embeds_recursively(false);
    exporter.run().expect("exporter returned error");

    assert_eq!(
        read_to_string("tests/testdata/expected/infinite-recursion/Note A.md").unwrap(),
        read_to_string(tmp_dir.path().join(PathBuf::from("Note A.md"))).unwrap(),
    );
}

#[test]
fn test_non_ascii_filenames() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");

    Exporter::new(
        PathBuf::from("tests/testdata/input/non-ascii/"),
        tmp_dir.path().to_path_buf(),
    )
    .run()
    .expect("exporter returned error");

    let walker = WalkDir::new("tests/testdata/expected/non-ascii/")
        // Without sorting here, different test runs may trigger the first assertion failure in
        // unpredictable order.
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter();
    for entry in walker {
        let entry = entry.unwrap();
        if entry.metadata().unwrap().is_dir() {
            continue;
        };
        let filename = entry.file_name().to_string_lossy().into_owned();
        let expected = read_to_string(entry.path()).unwrap_or_else(|_| {
            panic!(
                "failed to read {} from testdata/expected/non-ascii/",
                entry.path().display()
            )
        });
        let actual = read_to_string(tmp_dir.path().join(PathBuf::from(&filename)))
            .unwrap_or_else(|_| panic!("failed to read {} from temporary exportdir", filename));

        assert_eq!(
            expected, actual,
            "{} does not have expected content",
            filename
        );
    }
}

#[test]
fn test_same_filename_different_directories() {
    let tmp_dir = TempDir::new().expect("failed to make tempdir");
    Exporter::new(
        PathBuf::from("tests/testdata/input/same-filename-different-directories"),
        tmp_dir.path().to_path_buf(),
    )
    .run()
    .unwrap();

    let expected = if cfg!(windows) {
        read_to_string("tests/testdata/expected/same-filename-different-directories/Note.md")
            .unwrap()
            .replace('/', "\\")
    } else {
        read_to_string("tests/testdata/expected/same-filename-different-directories/Note.md")
            .unwrap()
    };

    let actual = read_to_string(tmp_dir.path().join(PathBuf::from("Note.md"))).unwrap();
    assert_eq!(expected, actual);
}
