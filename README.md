<!--

WARNING:

  Do not edit README.md directly, it is automatically generated from files in
  the docs directory.

  Instead of editing README.md, edit the corresponding Markdown files in the
  docs directory and run generate.sh. 

  To add new sections, create new files under docs and add these to _combined.md

-->


# Obsidian Export

*Obsidian Export is a CLI program and a Rust library to export an [Obsidian](https://obsidian.md/) vault to regular Markdown.*

* Recursively export Obsidian Markdown files to [CommonMark](https://commonmark.org/).
* Supports `[[note]]`-style references as well as `![[note]]` file includes.
* Support for [gitignore](https://git-scm.com/docs/gitignore)-style exclude patterns (default: `.export-ignore`).
* Automatically excludes files that are ignored by Git when the vault is located in a Git repository.
* Runs on all major platforms: Windows, Mac, Linux, BSDs.

Please note obsidian-export is not officially endorsed by the Obsidian team.
It supports most but not all of Obsidian's Markdown flavor.


# Installation

## Pre-built binaries

Binary releases for x86-64 processors are provided for Windows, Linux and Mac operating systems on a best-effort basis.
They are built with GitHub runners as part of the release workflow defined in `.github/workflows/release.yml`.

The resulting binaries can be downloaded from [https://github.com/zoni/obsidian-export/releases](https://github.com/zoni/obsidian-export/releases)

## Building from source

When binary releases are unavailable for your platform, or you do not trust the pre-built binaries, then *obsidian-export* can be compiled from source with relatively little effort.
This is done through [Cargo](https://doc.rust-lang.org/cargo/), the official package manager for Rust, with the following steps:

1. Install the Rust toolchain from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
1. Run: `cargo install obsidian-export`

 > 
 > It is expected that you successfully configured the PATH variable correctly while installing the Rust toolchain, as described under *"Configuring the PATH environment variable"* on [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

## Upgrading from earlier versions

If you downloaded a pre-built binary, upgrade by downloading the latest version to replace the old one.

If you built from source, upgrade by running `cargo install obsidian-export` again.


# Basic usage

The main interface of *obsidian-export* is the `obsidian-export` CLI command.
As a text interface, this must be run from a terminal or Windows PowerShell.

It is assumed that you have basic familiarity with command-line interfaces and that you set up your `PATH` correctly if you installed with `cargo`. 
Running `obsidian-export --version` should print a version number rather than giving some kind of error.

 > 
 > If you downloaded a pre-built binary and didn't put it a location referenced by `PATH` (for example, you put it in `Downloads`), you will need to provide the full path to the binary instead.
 > 
 > For example `~/Downloads/obsidian-export --version` on Mac/Linux or `~\Downloads\obsidian-export --version` on Windows (PowerShell).

In it's most basic form, `obsidian-export` takes just two mandatory arguments, a source and a destination:

````sh
obsidian-export /path/to/my-obsidian-vault /path/to/exported-notes/
````

This will export all of the files from `my-obsidian-vault` to `exported-notes`, except for those listed in `.export-ignore` or `.gitignore`.

 > 
 > Note that the destination directory must exist, so you may need to create a new, empty directory first.
 > 
 > If you give it an **existing** directory, files under that directory may get overwritten.

It is also possible to export individual files:

````sh
# Export as some-note.md to /tmp/export/
obsidian-export my-obsidian-vault/some-note.md /tmp/export/
# Export as exported-note.md in /tmp/
obsidian-export my-obsidian-vault/some-note.md /tmp/exported-note.md
````

## Character encodings

At present, UTF-8 character encoding is assumed for all note text as well as filenames.
All text and file handling performs [lossy conversion to Unicode strings](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy).

Use of non-UTF8 encodings may lead to issues like incorrect text replacement and failure to find linked notes.
While this may change in the future, there are no plans to change this behavior in the short term.


# Advanced usage

## Frontmatter

By default, frontmatter is copied over "as-is".

Some static site generators are picky about frontmatter and require it to be present.
Some get tripped up when Markdown files don't have frontmatter but start with a list item or horizontal rule.
In these cases, `--frontmatter=always` can be used to insert an empty frontmatter entry.

To completely remove any frontmatter from exported notes, use `--frontmatter=never`.

## Ignoring files

By default, hidden files, patterns listed in `.export-ignore` as well as any files ignored by git (if your vault is part of a git repository) will be excluded from exports.

These options may be adjusted with `--hidden`, `--ignore-file` and `--no-git` if desired.
(See `--help` for more information).

Notes linking to ignored notes will be unlinked (they'll only include the link text).
Embeds of ignored notes will be skipped entirely.

### Ignorefile syntax

The syntax for `.export-ignore` files is identical to that of [gitignore](https://git-scm.com/docs/gitignore) files.
Here's an example:

````
# Ignore the directory private that is located at the top of the export tree
/private
# Ignore any file or directory called `test`
test
# Ignore any PDF file
*.pdf
# ..but include special.pdf
!special.pdf
````

For more comprehensive documentation and examples, see the [gitignore](https://git-scm.com/docs/gitignore) manpage.

## Recursive embeds

It's possible to end up with "recursive embeds" when two notes embed each other.
This happens for example when a `Note A.md` contains `![[Note B]]` but `Note B.md` also contains `![[Note A]]`.

By default, this will trigger an error and display the chain of notes which caused the recursion.

This behavior may be changed by specifying `--no-recursive-embeds`.
Using this mode, if a note is encountered for a second time while processing the original note, instead of embedding it again a link to the note is inserted instead to break the cycle.


# Library usage

All of the functionality exposed by the `obsidian-export` CLI command is also accessible as a Rust library, exposed through the [`obsidian_export` crate](https://crates.io/crates/obsidian-export).

To get started, visit the library documentation on [obsidian_export](https://docs.rs/obsidian-export/latest/obsidian_export/) and [obsidian_export::Exporter](https://docs.rs/obsidian-export/latest/obsidian_export/struct.Exporter.html).


# License

Obsidian-export is dual-licensed under the [Apache 2.0](https://github.com/zoni/obsidian-export/blob/master/LICENSE-APACHE) and the [MIT](https://github.com/zoni/obsidian-export/blob/master/LICENSE-MIT) licenses.


# Changelog

## v0.6.0 (2021-02-15)

### New

* Add `--version` flag. \[Nick Groenen]

### Changes

* Don't Box FilterFn in WalkOptions. \[Nick Groenen]
  
  Previously, `filter_fn` on the `WalkOptions` struct looked like:
  
  ````
  pub filter_fn: Option<Box<&'static FilterFn>>,
  ````
  
  This boxing was unneccesary and has been changed to:
  
  ````
  pub filter_fn: Option<&'static FilterFn>,
  ````
  
  This will only affect people who use obsidian-export as a library in
  other Rust programs, not users of the CLI.
  
  For those library users, they no longer need to supply `FilterFn`
  wrapped in a Box.

### Fixes

* Recognize notes beginning with underscores. \[Nick Groenen]
  
  Notes with an underscore would fail to be recognized within Obsidian
  `[[_WikiLinks]]` due to the assumption that the underlying Markdown
  parser (pulldown_cmark) would emit the text between `[[` and `]]` as
  a single event.
  
  The note parser has now been rewritten to use a more reliable state
  machine which correctly recognizes this corner-case (and likely some
  others).

* Support self-references. \[Joshua Coles]
  
  This ensures links to headings within the same note (`[[#Heading]]`)
  resolve correctly.

### Other

* Avoid redundant "Release" in GitHub release titles. \[Nick Groenen]

* Add failing testcase for files with underscores. \[Nick Groenen]

* Add unit tests for display of ObsidianNoteReference. \[Nick Groenen]

* Add some unit tests for ObsidianNoteReference::from_str. \[Nick Groenen]

* Also run tests on pull requests. \[Nick Groenen]

* Apply clippy suggestions following rust 1.50.0. \[Nick Groenen]

* Fix infinite recursion bug with references to current file. \[Joshua Coles]

* Add tests for self-references. \[Joshua Coles]
  
  Note as there is no support for block references at the moment, the generated link goes nowhere, however it is to a reasonable ID

* Bump tempfile from 3.1.0 to 3.2.0. \[dependabot\[bot]]
  
  Bumps [tempfile](https://github.com/Stebalien/tempfile) from 3.1.0 to 3.2.0.
  
  * [Release notes](https://github.com/Stebalien/tempfile/releases)
  * [Changelog](https://github.com/Stebalien/tempfile/blob/master/NEWS)
  * [Commits](https://github.com/Stebalien/tempfile/commits)
* Bump eyre from 0.6.3 to 0.6.5. \[dependabot\[bot]]
  
  Bumps [eyre](https://github.com/yaahc/eyre) from 0.6.3 to 0.6.5.
  
  * [Release notes](https://github.com/yaahc/eyre/releases)
  * [Changelog](https://github.com/yaahc/eyre/blob/v0.6.5/CHANGELOG.md)
  * [Commits](https://github.com/yaahc/eyre/compare/v0.6.3...v0.6.5)
* Bump regex from 1.4.2 to 1.4.3. \[dependabot\[bot]]
  
  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.2 to 1.4.3.
  
  * [Release notes](https://github.com/rust-lang/regex/releases)
  * [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  * [Commits](https://github.com/rust-lang/regex/compare/1.4.2...1.4.3)

## v0.5.1 (2021-01-10)

### Fixes

* Find uppercased notes when referenced with lowercase. \[Nick Groenen]
  
  This commit fixes a bug where, if a note contained uppercase characters
  (for example `Note.md`) but was referred to using lowercase
  (`[[note]]`), that note would not be found.

## v0.5.0 (2021-01-05)

### New

* Add --no-recursive-embeds to break infinite recursion cycles. \[Nick Groenen]
  
  It's possible to end up with "recursive embeds" when two notes embed
  each other. This happens for example when a `Note A.md` contains
  `![[Note B]]` but `Note B.md` also contains `![[Note A]]`.
  
  By default, this will trigger an error and display the chain of notes
  which caused the recursion.
  
  Using the new `--no-recursive-embeds`, if a note is encountered for a
  second time while processing the original note, rather than embedding it
  again a link to the note is inserted instead to break the cycle.
  
  See also: https://github.com/zoni/obsidian-export/issues/1

* Make walk options configurable on CLI. \[Nick Groenen]
  
  By default hidden files, patterns listed in `.export-ignore` as well as
  any files ignored by git are excluded from exports. This behavior has
  been made configurable on the CLI using the new flags `--hidden`,
  `--ignore-file` and `--no-git`.

* Support links referencing headings. \[Nick Groenen]
  
  Previously, links referencing a heading (`[[note#heading]]`) would just
  link to the file name without including an anchor in the link target.
  Now, such references will include an appropriate `#anchor` attribute.
  
  Note that neither the original Markdown specification, nor the more
  recent CommonMark standard, specify how anchors should be constructed
  for a given heading.
  
  There are also some differences between the various Markdown rendering
  implementations.
  
  Obsidian-export uses the [slug](https://crates.io/crates/slug) crate to generate anchors which should
  be compatible with most implementations, however your mileage may vary.
  
  (For example, GitHub may leave a trailing `-` on anchors when headings
  end with a smiley. The slug library, and thus obsidian-export, will
  avoid such dangling dashes).

* Support embeds referencing headings. \[Nick Groenen]
  
  Previously, partial embeds (`![[note#heading]]`) would always include
  the entire file into the source note. Now, such embeds will only include
  the contents of the referenced heading (and any subheadings).
  
  Links and embeds of [arbitrary blocks](https://publish.obsidian.md/help/How+to/Link+to+blocks) remains unsupported at this time.

### Changes

* Print warnings to stderr rather than stdout. \[Nick Groenen]
  
  Warning messages emitted when encountering broken links/references will
  now be printed to stderr as opposed to stdout.

### Other

* Include filter_fn field in WalkOptions debug display. \[Nick Groenen]

## v0.4.0 (2020-12-23)

### Fixes

* Correct relative links within embedded notes. \[Nick Groenen]
  
  Links within an embedded note would point to other local resources
  relative to the filesystem location of the note being embedded.
  
  When a note inside a different directory would embed such a note, these
  links would point to invalid locations.
  
  Now these links are calculated relative to the top note, which ensures
  these links will point to the right path.

### Other

* Add brief library documentation to all public types and functions. \[Nick Groenen]

## v0.3.0 (2020-12-21)

### New

* Report file tree when RecursionLimitExceeded is hit. \[Nick Groenen]
  
  This refactors the Context to maintain a list of all the files which
  have been processed so far in a chain of embeds. This information is
  then used to print a more helpful error message to users of the CLI when
  RecursionLimitExceeded is returned.

### Changes

* Add extra whitespace around multi-line warnings. \[Nick Groenen]
  
  This makes errors a bit easier to distinguish after a number of warnings
  has been printed.

### Other

* Setup gitchangelog. \[Nick Groenen]
  
  This adds a changelog (CHANGES.md) which is automatically generated with
  [gitchangelog](https://github.com/vaab/gitchangelog).

## v0.2.0 (2020-12-13)

* Allow custom filter function to be passed with WalkOptions. \[Nick Groenen]

* Re-export vault_contents and WalkOptions as pub from crate root. \[Nick Groenen]

* Run mdbook hook against README.md too. \[Nick Groenen]

* Update installation instructions. \[Nick Groenen]
  
  Installation no longer requires a git repository URL now that a crate is
  published.

* Add MdBook generation script and precommit hook. \[Nick Groenen]

* Add more reliable non-ASCII tetscase. \[Nick Groenen]

* Create FUNDING.yml. \[Nick Groenen]

## v0.1.0 (2020-11-28)

* Public release. \[Nick Groenen]
