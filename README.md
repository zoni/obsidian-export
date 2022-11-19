<!--

WARNING:

  Do not edit README.md directly, it is automatically generated from files in
  the docs directory.

  Instead of editing README.md, edit the corresponding Markdown files in the
  docs directory and run generate.sh. 

  To add new sections, create new files under docs and add these to _combined.md

-->

# Obsidian Export

*Obsidian Export is a CLI program and a Rust library to export an [Obsidian] vault to regular Markdown.*

* Recursively export Obsidian Markdown files to [CommonMark].
* Supports `[[note]]`-style references as well as `![[note]]` file includes.
* Support for [gitignore]-style exclude patterns (default: `.export-ignore`).
* Automatically excludes files that are ignored by Git when the vault is located in a Git repository.
* Runs on all major platforms: Windows, Mac, Linux, BSDs.

Please note obsidian-export is not officially endorsed by the Obsidian team.
It supports most but not all of Obsidian's Markdown flavor.


# Installation

## Pre-built binaries

Binary releases for x86-64 processors are provided for Windows, Linux and Mac operating systems on a best-effort basis.
They are built with GitHub runners as part of the release workflow defined in `.github/workflows/release.yml`.

The resulting binaries can be downloaded from <https://github.com/zoni/obsidian-export/releases>

## Building from source

When binary releases are unavailable for your platform, or you do not trust the pre-built binaries, then *obsidian-export* can be compiled from source with relatively little effort.
This is done through [Cargo], the official package manager for Rust, with the following steps:

1. Install the Rust toolchain from <https://www.rust-lang.org/tools/install>
1. Run: `cargo install obsidian-export`

 > 
 > It is expected that you successfully configured the PATH variable correctly while installing the Rust toolchain, as described under *"Configuring the PATH environment variable"* on <https://www.rust-lang.org/tools/install>.

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

## Exporting notes

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

Note that in this mode, obsidian-export sees `some-note.md` as being the only file that exists in your vault so references to other notes won't be resolved.
This is by design.

If you'd like to export a single note while resolving links or embeds to other areas in your vault then you should instead specify the root of your vault as the source, passing the file you'd like to export with `--start-at`, as described in the next section.

### Exporting a partial vault

Using the `--start-at` argument, you can export just a subset of your vault.
Given the following vault structure:

````
my-obsidian-vault 
├── Notes/
├── Books/
└── People/
````

This will export only the notes in the `Books` directory to `exported-notes`:

````sh
obsidian-export my-obsidian-vault --start-at my-obsidian-vault/Books exported-notes
````

In this mode, all notes under the source (the first argument) are considered part of the vault so any references to these files will remain intact, even if they're not part of the exported notes.

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

The syntax for `.export-ignore` files is identical to that of [gitignore] files.
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

For more comprehensive documentation and examples, see the [gitignore] manpage.

## Recursive embeds

It's possible to end up with "recursive embeds" when two notes embed each other.
This happens for example when a `Note A.md` contains `![[Note B]]` but `Note B.md` also contains `![[Note A]]`.

By default, this will trigger an error and display the chain of notes which caused the recursion.

This behavior may be changed by specifying `--no-recursive-embeds`.
Using this mode, if a note is encountered for a second time while processing the original note, instead of embedding it again a link to the note is inserted instead to break the cycle.

## Relative links with Hugo

The [Hugo] static site generator [does not support relative links to files](https://notes.nick.groenen.me/notes/relative-linking-in-hugo/).
Instead, it expects you to link to other pages using the [`ref` and `relref` shortcodes].

As a result of this, notes that have been exported from Obsidian using obsidian-export do not work out of the box because Hugo doesn't resolve these links correctly.

[Markdown Render Hooks] (only supported using the default `goldmark` renderer) allow you to work around this issue however, making exported notes work with Hugo after a bit of one-time setup work.

Create the file `layouts/_default/_markup/render-link.html` with the following contents:

````
{{- $url := urls.Parse .Destination -}}
{{- $scheme := $url.Scheme -}}

<a href="
  {{- if eq $scheme "" -}}
    {{- if strings.HasSuffix $url.Path ".md" -}}
      {{- relref .Page .Destination | safeURL -}}
    {{- else -}}
      {{- .Destination | safeURL -}}
    {{- end -}}
  {{- else -}}
    {{- .Destination | safeURL -}}
  {{- end -}}"
  {{- with .Title }} title="{{ . | safeHTML }}"{{- end -}}>
  {{- .Text | safeHTML -}}
</a>

{{- /* whitespace stripped here to avoid trailing newline in rendered result caused by file EOL */ -}}
````

And `layouts/_default/_markup/render-image.html` for images:

````
{{- $url := urls.Parse .Destination -}}
{{- $scheme := $url.Scheme -}}

<img src="
  {{- if eq $scheme "" -}}
    {{- if strings.HasSuffix $url.Path ".md" -}}
      {{- relref .Page .Destination | safeURL -}}
    {{- else -}}
      {{- printf "/%s%s" .Page.File.Dir .Destination | safeURL -}}
    {{- end -}}
  {{- else -}}
    {{- .Destination | safeURL -}}
  {{- end -}}"
  {{- with .Title }} title="{{ . | safeHTML }}"{{- end -}}
  {{- with .Text }} alt="{{ . | safeHTML }}"
  {{- end -}}
/>

{{- /* whitespace stripped here to avoid trailing newline in rendered result caused by file EOL */ -}}
````

With these hooks in place, links to both notes as well as file attachments should now work correctly.

 > 
 > Note: If you're using a theme which comes with it's own render hooks, you might need to do a little extra work, or customize the snippets above, to avoid conflicts with the hooks from your theme.


# Library usage

All of the functionality exposed by the `obsidian-export` CLI command is also accessible as a Rust library, exposed through the [`obsidian_export` crate](https://crates.io/crates/obsidian-export).

To get started, visit the library documentation on [obsidian_export](https://docs.rs/obsidian-export/latest/obsidian_export/) and [obsidian_export::Exporter](https://docs.rs/obsidian-export/latest/obsidian_export/struct.Exporter.html).


# Contributing to Obsidian Export

Hi there!
Thank you so much for wanting to contribute to this project.
I greatly appreciate any efforts people like you put into making obsidian-export better!

Managing an open-source project can take a lot of time and effort however.
As this is a passion project which I maintain alongside my regular daytime job, I need to take some measures to safeguard my mental health and the enjoyment of this project.

This document aims to provide guidance which makes contributions easier by:

1. Defining the expectations I have of submissions to the codebase and the pull request process.
1. Helping you get set up for development on the code.
1. Providing pointers to some areas of the codebase, as well as some design considerations to take into account when making changes.

## Working with Rust

Obsidian-export is written in [Rust](https://www.rust-lang.org/), which is not the easiest of languages to master.
If you'd like to contribute but you don't know Rust, check out [Learn Rust](https://www.rust-lang.org/learn) for some suggestions of how to get started with the language.
In general, I will do my best to support you and help you out, but understand my time for mentoring is highly limited.

To work on the codebase, you'll also need the Rust toolchain, including cargo, rustfmt and clippy.
The easiest way is to [install Rust using rustup](https://www.rust-lang.org/tools/install), which lets you install rustfmt and clippy using `rustup component add rustfmt` and `rustup component add clippy` respectively.

## Design principles

My intention is to keep the core of `obsidian-export` as limited and small as possible, avoiding changes to the core [`Exporter`](https://docs.rs/obsidian-export/latest/obsidian_export/struct.Exporter.html) struct or any of its methods whenever possible.
This improves long-term maintainability and makes investigation of bugs simpler.

To keep the core of obsidian-export small while still supporting a wide range of use-cases, additional functionality should be pushed down into [postprocessors](https://docs.rs/obsidian-export/latest/obsidian_export/type.Postprocessor.html) as much as possible.
You can see some examples of this in:

* [Support Obsidian's "Strict line breaks" setting (#57)](https://github.com/zoni/obsidian-export/pull/57)
* [Frontmatter based filtering (#67)](https://github.com/zoni/obsidian-export/pull/67)

## Conventions

Code is formatted with [rustfmt](https://github.com/rust-lang/rustfmt) using the default options.
In addition, all default [clippy](https://github.com/rust-lang/rust-clippy) checks on the latest stable Rust compiler must also pass.
Both of these are enforced through CI using GitHub actions.

 > 
 > **💡 Tip: install pre-commit hooks** 
 > 
 > This codebase is set up with the [pre-commit framework](https://pre-commit.com/) to automatically run the appropriate checks locally whenever you commit.
 > Assuming you [have pre-commit installed](https://pre-commit.com/#install), all you need to do is run `pre-commit install` once to get this set up.

Following my advice on [creating high-quality commits](https://nick.groenen.me/notes/high-quality-commits/) will make it easier for me to review changes.
I don't insist on this, but pull requests which fail to adhere to these conventions are at risk of being squashed and having their commit messages rewritten when they are accepted.

## Tests

In order to have confidence that your changes work as intended, as well as to avoid regressions when making changes in the future, I would like to see code accompanied by test cases.

At the moment, the test framework primary relies on high-level integration tests, all of which are defined in the [tests](tests/) directory.
These rely on comparing Markdown notes [before](tests/testdata/input) and [after](tests/testdata/expected) running an export.
By studying some of the existing tests, you should be able to copy and adapt these for your own changes.

For an example of doing low-level unit tests, you can look at the end of [frontmatter.rs](src/frontmatter.rs).

## Documentation

I place a lot of value on good documentation and would encourage you to include updates to the docs with your changes.
Changes or additions to public methods and attributes **must** come with proper documentation for a PR to be accepted.

Advice on writing Rust documentation can be found in:

* [The rustdoc book: How to write documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
* [Rust by example: Documentation](https://doc.rust-lang.org/rust-by-example/meta/doc.html)

Updates to the user guide/README instructions are also preferred, but optional.
If you don't feel comfortable writing user documentation, I will be happy to guide you or do it for you.

 > 
 > **⚠ Warning**
 > 
 > If you update the README file, take note that you must edit the fragments in the [docs](docs/) directory as opposed to the README in the root of the repository, which is auto-generated.


# License

Obsidian-export is dual-licensed under the [Apache 2.0] and the [MIT] licenses.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.


# Changelog

## v22.11.0 (2022-11-19)

### New

* Apply unicode normalization while resolving notes. \[Nick Groenen\]
  
  The unicode standard allows for certain (visually) identical characters to
  be represented in different ways.
  
  For example the character ä may be represented as a single combined
  codepoint "Latin Small Letter A with Diaeresis" (U+00E4) or by the
  combination of "Latin Small Letter A" (U+0061) followed by "Combining
  Diaeresis" (U+0308).
  
  When encoded with UTF-8, these are represented as respectively the two
  bytes 0xC3 0xA4, and the three bytes 0x61 0xCC 0x88.
  
  A user linking to notes with these characters in their titles would
  expect these two variants to link to the same file, given they are
  visually identical and have the exact same semantic meaning.
  
  The unicode standard defines a method to deconstruct and normalize these
  forms, so that a byte comparison on the normalized forms of these
  variants ends up comparing the same thing. This is called Unicode
  Normalization, defined in Unicode® Standard Annex #15
  (http://www.unicode.org/reports/tr15/).
  
  The W3C Working Group has written an excellent explanation of the
  problems regarding string matching, and how unicode normalization helps
  with this process: https://www.w3.org/TR/charmod-norm/#unicodeNormalization
  
  With this change, obsidian-export will perform unicode normalization
  (specifically the C (or NFC) normalization form) on all note titles
  while looking up link references, ensuring visually identical links are
  treated as being similar, even if they were encoded as different
  variants.
  
  A special thanks to Hans Raaf (@oderwat) for reporting and helping track
  down this issue.

### Breaking Changes (affects library API only)

* Pass context and events as mutable references to postprocessors. \[Nick Groenen\]
  
  Instead of passing clones of context and the markdown tree to
  postprocessors, pass them a mutable reference which may be modified
  in-place.
  
  This is a breaking change to the postprocessor implementation, changing
  both the input arguments as well as the return value:
  
  ````diff
  -    dyn Fn(Context, MarkdownEvents) -> (Context, MarkdownEvents, PostprocessorResult) + Send + Sync;
  +    dyn Fn(&mut Context, &mut MarkdownEvents) -> PostprocessorResult + Send + Sync;
  ````
  
  With this change the postprocessor API becomes a little more ergonomic
  to use however, especially making the intent around return statements more clear.

### Other

* Use path.Join to construct hugo links (#92) \[Chang-Yen Tseng\]
  
  Use path.Join so that it will render correctly on Windows
  (path.Join will convert Windows backslash to forward slash)

* Bump crossbeam-utils from 0.8.5 to 0.8.12. \[dependabot\[bot\]\]
  
  Bumps [crossbeam-utils](https://github.com/crossbeam-rs/crossbeam) from 0.8.5 to 0.8.12.
  
  * [Release notes](https://github.com/crossbeam-rs/crossbeam/releases)
  * [Changelog](https://github.com/crossbeam-rs/crossbeam/blob/master/CHANGELOG.md)
  * [Commits](https://github.com/crossbeam-rs/crossbeam/compare/crossbeam-utils-0.8.5...crossbeam-utils-0.8.12)
  ---
  
  updated-dependencies:
  
  * dependency-name: crossbeam-utils
    dependency-type: indirect
    ...
* Bump regex from 1.6.0 to 1.7.0. \[dependabot\[bot\]\]
  
  Bumps [regex](https://github.com/rust-lang/regex) from 1.6.0 to 1.7.0.
  
  * [Release notes](https://github.com/rust-lang/regex/releases)
  * [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  * [Commits](https://github.com/rust-lang/regex/compare/1.6.0...1.7.0)
  ---
  
  updated-dependencies:
  
  * dependency-name: regex
    dependency-type: direct:production
    update-type: version-update:semver-minor
    ...
* Bump actions/checkout from 2 to 3. \[dependabot\[bot\]\]
  
  Bumps [actions/checkout](https://github.com/actions/checkout) from 2 to 3.
  
  * [Release notes](https://github.com/actions/checkout/releases)
  * [Changelog](https://github.com/actions/checkout/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/actions/checkout/compare/v2...v3)
  ---
  
  updated-dependencies:
  
  * dependency-name: actions/checkout
    dependency-type: direct:production
    update-type: version-update:semver-major
    ...
* Bump actions/upload-artifact from 2 to 3. \[dependabot\[bot\]\]
  
  Bumps [actions/upload-artifact](https://github.com/actions/upload-artifact) from 2 to 3.
  
  * [Release notes](https://github.com/actions/upload-artifact/releases)
  * [Commits](https://github.com/actions/upload-artifact/compare/v2...v3)
  ---
  
  updated-dependencies:
  
  * dependency-name: actions/upload-artifact
    dependency-type: direct:production
    update-type: version-update:semver-major
    ...
* Bump thread_local from 1.1.3 to 1.1.4. \[dependabot\[bot\]\]
  
  Bumps [thread_local](https://github.com/Amanieu/thread_local-rs) from 1.1.3 to 1.1.4.
  
  * [Release notes](https://github.com/Amanieu/thread_local-rs/releases)
  * [Commits](https://github.com/Amanieu/thread_local-rs/compare/v1.1.3...1.1.4)
  ---
  
  updated-dependencies:
  
  * dependency-name: thread_local
    dependency-type: indirect
    ...
* Remove needless borrows. \[Nick Groenen\]

* Upgrade snafu to 0.7.x. \[Nick Groenen\]

* Upgrade pulldown-cmark-to-cmark to 10.0.x. \[Nick Groenen\]

* Upgrade serde_yaml to 0.9.x. \[Nick Groenen\]

* Upgrade minor dependencies. \[Nick Groenen\]

* Fix new clippy lints. \[Nick Groenen\]

* Add a contributor guide. \[Nick Groenen\]

* Simplify pre-commit setup. \[Nick Groenen\]
  
  No need to depend on a third-party hook repository when each of these
  checks is easily defined and run through system commands.
  
  This also allows us to actually run tests, which is current unsupported
  (https://github.com/doublify/pre-commit-rust/pull/19)

* Bump tempfile from 3.2.0 to 3.3.0. \[dependabot\[bot\]\]
  
  Bumps [tempfile](https://github.com/Stebalien/tempfile) from 3.2.0 to 3.3.0.
  
  * [Release notes](https://github.com/Stebalien/tempfile/releases)
  * [Changelog](https://github.com/Stebalien/tempfile/blob/master/NEWS)
  * [Commits](https://github.com/Stebalien/tempfile/compare/v3.2.0...v3.3.0)
  ---
  
  updated-dependencies:
  
  * dependency-name: tempfile
    dependency-type: direct:production
    update-type: version-update:semver-minor
    ...

## v22.1.0 (2022-01-02)

Happy new year! On this second day of 2022 comes a fresh release with one
notable new feature.

### New

* Support Obsidian's "Strict line breaks" setting. \[Nick Groenen\]
  
  This change introduces a new `--hard-linebreaks` CLI argument. When
  used, this converts soft line breaks to hard line breaks, mimicking
  Obsidian's "Strict line breaks" setting.
  
   > 
   > Implementation detail: I considered naming this flag
   > `--strict-line-breaks` to be consistent with Obsidian itself, however I
   > feel the name is somewhat misleading and ill-chosen.

### Other

* Give release binaries file extensions. \[Nick Groenen\]
  
  This may make it more clear to users that these are precompiled, binary
  files. This is especially relevant on Windows, where the convention is
  that executable files have a `.exe` extension, as seen in #49.

* Upgrade dependencies. \[Nick Groenen\]
  
  This commit upgrades all dependencies to their current latest versions. Most
  notably, this includes upgrades to the following most critical libraries:
  
  ````
  pulldown-cmark v0.8.0 -> v0.9.0
  pulldown-cmark-to-cmark v7.1.1 -> v9.0.0
  ````
  
  In total, these dependencies were upgraded:
  
  ````
  bstr v0.2.16 -> v0.2.17
  ignore v0.4.17 -> v0.4.18
  libc v0.2.101 -> v0.2.112
  memoffset v0.6.4 -> v0.6.5
  num_cpus v1.13.0 -> v1.13.1
  once_cell v1.8.0 -> v1.9.0
  ppv-lite86 v0.2.10 -> v0.2.16
  proc-macro2 v1.0.29 -> v1.0.36
  pulldown-cmark v0.8.0 -> v0.9.0
  pulldown-cmark-to-cmark v7.1.1 -> v9.0.0
  quote v1.0.9 -> v1.0.14
  rayon v1.5.0 -> v1.5.1
  regex v1.5.3 -> v1.5.4
  serde v1.0.130 -> v1.0.132
  syn v1.0.75 -> v1.0.84
  unicode-width v0.1.8 -> v0.1.9
  version_check v0.9.3 -> v0.9.4
  ````

* Bump serde_yaml from 0.8.21 to 0.8.23 (#52) \[dependabot\[bot\]\]
  
  Bumps [serde_yaml](https://github.com/dtolnay/serde-yaml) from 0.8.21 to 0.8.23.
  
  * [Release notes](https://github.com/dtolnay/serde-yaml/releases)
  * [Commits](https://github.com/dtolnay/serde-yaml/compare/0.8.21...0.8.23)
  ---
  
  updated-dependencies:
  
  * dependency-name: serde_yaml
    dependency-type: direct:production
    update-type: version-update:semver-patch
    ...
* Bump pulldown-cmark-to-cmark from 7.1.0 to 7.1.1 (#51) \[dependabot\[bot\]\]
  
  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 7.1.0 to 7.1.1.
  
  * [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  * [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v7.1.0...v7.1.1)
  ---
  
  updated-dependencies:
  
  * dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-patch
    ...
* Bump pulldown-cmark-to-cmark from 7.0.0 to 7.1.0 (#48) \[dependabot\[bot\]\]
  
  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 7.0.0 to 7.1.0.
  
  * [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  * [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v7.0.0...v7.1.0)
  ---
  
  updated-dependencies:
  
  * dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-minor
    ...
* Bump pulldown-cmark-to-cmark from 6.0.4 to 7.0.0 (#47) \[dependabot\[bot\]\]
  
  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 6.0.4 to 7.0.0.
  
  * [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  * [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v6.0.4...v7.0.0)
  ---
  
  updated-dependencies:
  
  * dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-major
    ...
* Bump pathdiff from 0.2.0 to 0.2.1 (#46) \[dependabot\[bot\]\]
  
  Bumps [pathdiff](https://github.com/Manishearth/pathdiff) from 0.2.0 to 0.2.1.
  
  * [Release notes](https://github.com/Manishearth/pathdiff/releases)
  * [Commits](https://github.com/Manishearth/pathdiff/commits)
  ---
  
  updated-dependencies:
  
  * dependency-name: pathdiff
    dependency-type: direct:production
    update-type: version-update:semver-patch
    ...
* Bump pulldown-cmark-to-cmark from 6.0.3 to 6.0.4 (#44) \[dependabot\[bot\]\]
  
  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 6.0.3 to 6.0.4.
  
  * [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  * [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v6.0.3...v6.0.4)
  ---
  
  updated-dependencies:
  
  * dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-patch
    ...
* Bump pretty_assertions from 0.7.2 to 1.0.0 (#45) \[dependabot\[bot\]\]
  
  Bumps [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) from 0.7.2 to 1.0.0.
  
  * [Release notes](https://github.com/colin-kiegel/rust-pretty-assertions/releases)
  * [Changelog](https://github.com/colin-kiegel/rust-pretty-assertions/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/colin-kiegel/rust-pretty-assertions/compare/v0.7.2...v1.0.0)
  ---
  
  updated-dependencies:
  
  * dependency-name: pretty_assertions
    dependency-type: direct:production
    update-type: version-update:semver-major
    ...

## v21.9.1 (2021-09-24)

### Changes

* Treat SVG files as embeddable images. \[Narayan Sainaney\]
  
  This will ensure SVG files are included as an image when using `![[foo.svg]]` syntax, as opposed to only being linked to.

### Other

* Bump pulldown-cmark-to-cmark from 6.0.2 to 6.0.3. \[dependabot\[bot\]\]
  
  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 6.0.2 to 6.0.3.
  
  * [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  * [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v6.0.2...v6.0.3)
  ---
  
  updated-dependencies:
  
  * dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-patch
    ...
* Bump serde_yaml from 0.8.20 to 0.8.21. \[dependabot\[bot\]\]
  
  Bumps [serde_yaml](https://github.com/dtolnay/serde-yaml) from 0.8.20 to 0.8.21.
  
  * [Release notes](https://github.com/dtolnay/serde-yaml/releases)
  * [Commits](https://github.com/dtolnay/serde-yaml/compare/0.8.20...0.8.21)
  ---
  
  updated-dependencies:
  
  * dependency-name: serde_yaml
    dependency-type: direct:production
    update-type: version-update:semver-patch
    ...

## v21.9.0 (2021-09-12)

 > 
 > This release switches to a [calendar versioning scheme](https://calver.org/overview.html).
 > Details on this decision can be read in [switching obsidian-export to CalVer](https://nick.groenen.me/posts/switching-obsidian-export-to-calver/).

### New

* Support postprocessors running on embedded notes. \[Nick Groenen\]
  
  This introduces support for postprocessors that are run on the result of
  a note that is being embedded into another note. This differs from the
  existing postprocessors (which remain unchanged) that run once all
  embeds have been processed and merged with the final note.
  
  These "embed postprocessors" may be set through the new
  `Exporter::add_embed_postprocessor` method.

* Add start_at option to export a partial vault. \[Nick Groenen\]
  
  This introduces a new `--start-at` CLI argument and corresponding
  `start_at()` method on the Exporter type that allows exporting of only a
  given subdirectory within a vault.
  
  See the updated README file for more details on when and how this may be
  used.

### Other

* Don't build docs for the bin target. \[Nick Groenen\]
  
  The library contains documentation covering both CLI and library usage,
  there's no separate documentation for just the binary target.

* Move postprocessor tests into their own file for clarity. \[Nick Groenen\]

* Update indirect dependencies. \[Nick Groenen\]

* Bump serde_yaml from 0.8.19 to 0.8.20. \[dependabot\[bot\]\]
  
  Bumps [serde_yaml](https://github.com/dtolnay/serde-yaml) from 0.8.19 to 0.8.20.
  
  * [Release notes](https://github.com/dtolnay/serde-yaml/releases)
  * [Commits](https://github.com/dtolnay/serde-yaml/compare/0.8.19...0.8.20)
  ---
  
  updated-dependencies:
  
  * dependency-name: serde_yaml
    dependency-type: direct:production
    update-type: version-update:semver-patch
    ...
* Don't borrow references that are immediately dereferenced. \[Nick Groenen\]
  
  This was caught by a recently introduced clippy rule

* Bump serde_yaml from 0.8.17 to 0.8.19. \[dependabot\[bot\]\]
  
  Bumps [serde_yaml](https://github.com/dtolnay/serde-yaml) from 0.8.17 to 0.8.19.
  
  * [Release notes](https://github.com/dtolnay/serde-yaml/releases)
  * [Commits](https://github.com/dtolnay/serde-yaml/compare/0.8.17...0.8.19)
  ---
  
  updated-dependencies:
  
  * dependency-name: serde_yaml
    dependency-type: direct:production
    update-type: version-update:semver-patch
    ...
* Update dependencies. \[Nick Groenen\]

* Fix 4 new clippy lints. \[Nick Groenen\]

* Bump regex from 1.4.6 to 1.5.3. \[dependabot\[bot\]\]
  
  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.6 to 1.5.3.
  
  * [Release notes](https://github.com/rust-lang/regex/releases)
  * [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  * [Commits](https://github.com/rust-lang/regex/compare/1.4.6...1.5.3)
* Bump pretty_assertions from 0.7.1 to 0.7.2. \[dependabot\[bot\]\]
  
  Bumps [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) from 0.7.1 to 0.7.2.
  
  * [Release notes](https://github.com/colin-kiegel/rust-pretty-assertions/releases)
  * [Changelog](https://github.com/colin-kiegel/rust-pretty-assertions/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/colin-kiegel/rust-pretty-assertions/compare/v0.7.1...v0.7.2)
* Bump regex from 1.4.5 to 1.4.6. \[dependabot\[bot\]\]
  
  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.5 to 1.4.6.
  
  * [Release notes](https://github.com/rust-lang/regex/releases)
  * [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  * [Commits](https://github.com/rust-lang/regex/compare/1.4.5...1.4.6)

## v0.7.0 (2021-04-11)

### New

* Postprocessing support. \[Nick Groenen\]
  
  Add support for postprocessing of Markdown prior to writing converted
  notes to disk.
  
  Postprocessors may be used when making use of Obsidian export as a Rust
  library to do the following:
  
  1. Modify a note's `Context`, for example to change the destination
     filename or update its Frontmatter.
  1. Change a note's contents by altering `MarkdownEvents`.
  1. Prevent later postprocessors from running or cause a note to be
     skipped entirely.
  Future releases of Obsidian export may come with built-in postprocessors
  for users of the command-line tool to use, if general use-cases can be
  identified.
  
  For example, a future release might include functionality to make notes
  more suitable for the Hugo static site generator. This functionality
  would be implemented as a postprocessor that could be enabled through
  command-line flags.

### Fixes

* Also percent-encode `?` in filenames. \[Nick Groenen\]
  
  A recent Obsidian update expanded the list of allowed characters in
  filenames, which now includes `?` as well. This needs to be
  percent-encoded for proper links in static site generators like Hugo.

### Other

* Bump pretty_assertions from 0.6.1 to 0.7.1. \[dependabot\[bot\]\]
  
  Bumps [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) from 0.6.1 to 0.7.1.
  
  * [Release notes](https://github.com/colin-kiegel/rust-pretty-assertions/releases)
  * [Changelog](https://github.com/colin-kiegel/rust-pretty-assertions/blob/main/CHANGELOG.md)
  * [Commits](https://github.com/colin-kiegel/rust-pretty-assertions/compare/v0.6.1...v0.7.1)
* Bump walkdir from 2.3.1 to 2.3.2. \[dependabot\[bot\]\]
  
  Bumps [walkdir](https://github.com/BurntSushi/walkdir) from 2.3.1 to 2.3.2.
  
  * [Release notes](https://github.com/BurntSushi/walkdir/releases)
  * [Commits](https://github.com/BurntSushi/walkdir/compare/2.3.1...2.3.2)
* Bump regex from 1.4.3 to 1.4.5. \[dependabot\[bot\]\]
  
  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.3 to 1.4.5.
  
  * [Release notes](https://github.com/rust-lang/regex/releases)
  * [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  * [Commits](https://github.com/rust-lang/regex/compare/1.4.3...1.4.5)

## v0.6.0 (2021-02-15)

### New

* Add `--version` flag. \[Nick Groenen\]

### Changes

* Don't Box FilterFn in WalkOptions. \[Nick Groenen\]
  
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

* Recognize notes beginning with underscores. \[Nick Groenen\]
  
  Notes with an underscore would fail to be recognized within Obsidian
  `[[_WikiLinks]]` due to the assumption that the underlying Markdown
  parser (pulldown_cmark) would emit the text between `[[` and `]]` as
  a single event.
  
  The note parser has now been rewritten to use a more reliable state
  machine which correctly recognizes this corner-case (and likely some
  others).

* Support self-references. \[Joshua Coles\]
  
  This ensures links to headings within the same note (`[[#Heading]]`)
  resolve correctly.

### Other

* Avoid redundant "Release" in GitHub release titles. \[Nick Groenen\]

* Add failing testcase for files with underscores. \[Nick Groenen\]

* Add unit tests for display of ObsidianNoteReference. \[Nick Groenen\]

* Add some unit tests for ObsidianNoteReference::from_str. \[Nick Groenen\]

* Also run tests on pull requests. \[Nick Groenen\]

* Apply clippy suggestions following rust 1.50.0. \[Nick Groenen\]

* Fix infinite recursion bug with references to current file. \[Joshua Coles\]

* Add tests for self-references. \[Joshua Coles\]
  
  Note as there is no support for block references at the moment, the generated link goes nowhere, however it is to a reasonable ID

* Bump tempfile from 3.1.0 to 3.2.0. \[dependabot\[bot\]\]
  
  Bumps [tempfile](https://github.com/Stebalien/tempfile) from 3.1.0 to 3.2.0.
  
  * [Release notes](https://github.com/Stebalien/tempfile/releases)
  * [Changelog](https://github.com/Stebalien/tempfile/blob/master/NEWS)
  * [Commits](https://github.com/Stebalien/tempfile/commits)
* Bump eyre from 0.6.3 to 0.6.5. \[dependabot\[bot\]\]
  
  Bumps [eyre](https://github.com/yaahc/eyre) from 0.6.3 to 0.6.5.
  
  * [Release notes](https://github.com/yaahc/eyre/releases)
  * [Changelog](https://github.com/yaahc/eyre/blob/v0.6.5/CHANGELOG.md)
  * [Commits](https://github.com/yaahc/eyre/compare/v0.6.3...v0.6.5)
* Bump regex from 1.4.2 to 1.4.3. \[dependabot\[bot\]\]
  
  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.2 to 1.4.3.
  
  * [Release notes](https://github.com/rust-lang/regex/releases)
  * [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  * [Commits](https://github.com/rust-lang/regex/compare/1.4.2...1.4.3)

## v0.5.1 (2021-01-10)

### Fixes

* Find uppercased notes when referenced with lowercase. \[Nick Groenen\]
  
  This commit fixes a bug where, if a note contained uppercase characters
  (for example `Note.md`) but was referred to using lowercase
  (`[[note]]`), that note would not be found.

## v0.5.0 (2021-01-05)

### New

* Add --no-recursive-embeds to break infinite recursion cycles. \[Nick Groenen\]
  
  It's possible to end up with "recursive embeds" when two notes embed
  each other. This happens for example when a `Note A.md` contains
  `![[Note B]]` but `Note B.md` also contains `![[Note A]]`.
  
  By default, this will trigger an error and display the chain of notes
  which caused the recursion.
  
  Using the new `--no-recursive-embeds`, if a note is encountered for a
  second time while processing the original note, rather than embedding it
  again a link to the note is inserted instead to break the cycle.
  
  See also: https://github.com/zoni/obsidian-export/issues/1

* Make walk options configurable on CLI. \[Nick Groenen\]
  
  By default hidden files, patterns listed in `.export-ignore` as well as
  any files ignored by git are excluded from exports. This behavior has
  been made configurable on the CLI using the new flags `--hidden`,
  `--ignore-file` and `--no-git`.

* Support links referencing headings. \[Nick Groenen\]
  
  Previously, links referencing a heading (`[[note#heading]]`) would just
  link to the file name without including an anchor in the link target.
  Now, such references will include an appropriate `#anchor` attribute.
  
  Note that neither the original Markdown specification, nor the more
  recent CommonMark standard, specify how anchors should be constructed
  for a given heading.
  
  There are also some differences between the various Markdown rendering
  implementations.
  
  Obsidian-export uses the [slug] crate to generate anchors which should
  be compatible with most implementations, however your mileage may vary.
  
  (For example, GitHub may leave a trailing `-` on anchors when headings
  end with a smiley. The slug library, and thus obsidian-export, will
  avoid such dangling dashes).

* Support embeds referencing headings. \[Nick Groenen\]
  
  Previously, partial embeds (`![[note#heading]]`) would always include
  the entire file into the source note. Now, such embeds will only include
  the contents of the referenced heading (and any subheadings).
  
  Links and embeds of [arbitrary blocks] remains unsupported at this time.

### Changes

* Print warnings to stderr rather than stdout. \[Nick Groenen\]
  
  Warning messages emitted when encountering broken links/references will
  now be printed to stderr as opposed to stdout.

### Other

* Include filter_fn field in WalkOptions debug display. \[Nick Groenen\]

## v0.4.0 (2020-12-23)

### Fixes

* Correct relative links within embedded notes. \[Nick Groenen\]
  
  Links within an embedded note would point to other local resources
  relative to the filesystem location of the note being embedded.
  
  When a note inside a different directory would embed such a note, these
  links would point to invalid locations.
  
  Now these links are calculated relative to the top note, which ensures
  these links will point to the right path.

### Other

* Add brief library documentation to all public types and functions. \[Nick Groenen\]

## v0.3.0 (2020-12-21)

### New

* Report file tree when RecursionLimitExceeded is hit. \[Nick Groenen\]
  
  This refactors the Context to maintain a list of all the files which
  have been processed so far in a chain of embeds. This information is
  then used to print a more helpful error message to users of the CLI when
  RecursionLimitExceeded is returned.

### Changes

* Add extra whitespace around multi-line warnings. \[Nick Groenen\]
  
  This makes errors a bit easier to distinguish after a number of warnings
  has been printed.

### Other

* Setup gitchangelog. \[Nick Groenen\]
  
  This adds a changelog (CHANGES.md) which is automatically generated with
  [gitchangelog].

## v0.2.0 (2020-12-13)

* Allow custom filter function to be passed with WalkOptions. \[Nick Groenen\]

* Re-export vault_contents and WalkOptions as pub from crate root. \[Nick Groenen\]

* Run mdbook hook against README.md too. \[Nick Groenen\]

* Update installation instructions. \[Nick Groenen\]
  
  Installation no longer requires a git repository URL now that a crate is
  published.

* Add MdBook generation script and precommit hook. \[Nick Groenen\]

* Add more reliable non-ASCII tetscase. \[Nick Groenen\]

* Create FUNDING.yml. \[Nick Groenen\]

## v0.1.0 (2020-11-28)

* Public release. \[Nick Groenen\]

[Obsidian]: https://obsidian.md/
[CommonMark]: https://commonmark.org/
[gitignore]: https://git-scm.com/docs/gitignore
[Cargo]: https://doc.rust-lang.org/cargo/
[Hugo]: https://gohugo.io
[`ref` and `relref` shortcodes]: https://gohugo.io/content-management/cross-references/
[Markdown Render Hooks]: https://gohugo.io/getting-started/configuration-markup#markdown-render-hooks
[Apache 2.0]: https://github.com/zoni/obsidian-export/blob/master/LICENSE-APACHE
[MIT]: https://github.com/zoni/obsidian-export/blob/master/LICENSE-MIT
[slug]: https://crates.io/crates/slug
[arbitrary blocks]: https://publish.obsidian.md/help/How+to/Link+to+blocks
[gitchangelog]: https://github.com/vaab/gitchangelog
