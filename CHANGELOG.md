# Changelog

## v23.12.0 (2023-12-03)

### New

- Implement frontmatter based filtering (#163) [Martin Heuschober]

  This allows limiting the notes that will be exported using `--skip-tags` and `--only-tags`:

  - using `--skip-tags foo --skip-tags bar` will skip any files that have the tags `foo` or `bar` in their frontmatter
  - using `--only-tags foo --only-tags bar` will skip any files that **don't** have the tags `foo` or `bar` in their frontmatter

### Fixes

- Trim filenames while resolving wikilinks [Nick Groenen]

  Obsidian trims the filename part in a [[WikiLink|label]], so each of
  these are equivalent:

  ```
  [[wikilink]]
  [[ wikilink ]]
  [[ wikilink |wikilink]]
  ```

  Obsidian-export now behaves similarly.

  Fixes #188

### Other

- Relicense to BSD-2-Clause Plus Patent License [Nick Groenen]

  This license achieves everything that dual-licensing under MIT + Apache
  aims for, but without the weirdness of being under two licenses.

  Having checked external contributions, I feel pretty confident that I
  can unilaterally make this license change, as people have only
  contributed a handful of one-line changes of no significance towards
  copyrighted work up to this point.


- Add a lifetime annotation to the Postprocesor type [Robert Sesek]

  This lets the compiler reason about the lifetimes of objects used by the
  postprocessor, if the callback captures variables.

  See zoni/obsidian-export#175

- Use cargo-dist to create release artifacts [Nick Groenen]

  This will create binaries for more platforms (including ARM builds for
  MacOS) and installer scripts in addition to just the binaries themselves.

## v22.11.0 (2022-11-19)

### New

* Apply unicode normalization while resolving notes. [Nick Groenen]

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

* Pass context and events as mutable references to postprocessors. [Nick Groenen]

  Instead of passing clones of context and the markdown tree to
  postprocessors, pass them a mutable reference which may be modified
  in-place.

  This is a breaking change to the postprocessor implementation, changing
  both the input arguments as well as the return value:

  ```diff
  -    dyn Fn(Context, MarkdownEvents) -> (Context, MarkdownEvents, PostprocessorResult) + Send + Sync;
  +    dyn Fn(&mut Context, &mut MarkdownEvents) -> PostprocessorResult + Send + Sync;
  ```

  With this change the postprocessor API becomes a little more ergonomic
  to use however, especially making the intent around return statements more clear.

### Other

* Use path.Join to construct hugo links (#92) [Chang-Yen Tseng]

  Use path.Join so that it will render correctly on Windows
  (path.Join will convert Windows backslash to forward slash)

* Bump crossbeam-utils from 0.8.5 to 0.8.12. [dependabot[bot]]

  Bumps [crossbeam-utils](https://github.com/crossbeam-rs/crossbeam) from 0.8.5 to 0.8.12.
  - [Release notes](https://github.com/crossbeam-rs/crossbeam/releases)
  - [Changelog](https://github.com/crossbeam-rs/crossbeam/blob/master/CHANGELOG.md)
  - [Commits](https://github.com/crossbeam-rs/crossbeam/compare/crossbeam-utils-0.8.5...crossbeam-utils-0.8.12)

  ---
  updated-dependencies:
  - dependency-name: crossbeam-utils
    dependency-type: indirect
  ...

* Bump regex from 1.6.0 to 1.7.0. [dependabot[bot]]

  Bumps [regex](https://github.com/rust-lang/regex) from 1.6.0 to 1.7.0.
  - [Release notes](https://github.com/rust-lang/regex/releases)
  - [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  - [Commits](https://github.com/rust-lang/regex/compare/1.6.0...1.7.0)

  ---
  updated-dependencies:
  - dependency-name: regex
    dependency-type: direct:production
    update-type: version-update:semver-minor
  ...

* Bump actions/checkout from 2 to 3. [dependabot[bot]]

  Bumps [actions/checkout](https://github.com/actions/checkout) from 2 to 3.
  - [Release notes](https://github.com/actions/checkout/releases)
  - [Changelog](https://github.com/actions/checkout/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/actions/checkout/compare/v2...v3)

  ---
  updated-dependencies:
  - dependency-name: actions/checkout
    dependency-type: direct:production
    update-type: version-update:semver-major
  ...

* Bump actions/upload-artifact from 2 to 3. [dependabot[bot]]

  Bumps [actions/upload-artifact](https://github.com/actions/upload-artifact) from 2 to 3.
  - [Release notes](https://github.com/actions/upload-artifact/releases)
  - [Commits](https://github.com/actions/upload-artifact/compare/v2...v3)

  ---
  updated-dependencies:
  - dependency-name: actions/upload-artifact
    dependency-type: direct:production
    update-type: version-update:semver-major
  ...

* Bump thread_local from 1.1.3 to 1.1.4. [dependabot[bot]]

  Bumps [thread_local](https://github.com/Amanieu/thread_local-rs) from 1.1.3 to 1.1.4.
  - [Release notes](https://github.com/Amanieu/thread_local-rs/releases)
  - [Commits](https://github.com/Amanieu/thread_local-rs/compare/v1.1.3...1.1.4)

  ---
  updated-dependencies:
  - dependency-name: thread_local
    dependency-type: indirect
  ...

* Remove needless borrows. [Nick Groenen]

* Upgrade snafu to 0.7.x. [Nick Groenen]

* Upgrade pulldown-cmark-to-cmark to 10.0.x. [Nick Groenen]

* Upgrade serde_yaml to 0.9.x. [Nick Groenen]

* Upgrade minor dependencies. [Nick Groenen]

* Fix new clippy lints. [Nick Groenen]

* Add a contributor guide. [Nick Groenen]

* Simplify pre-commit setup. [Nick Groenen]

  No need to depend on a third-party hook repository when each of these
  checks is easily defined and run through system commands.

  This also allows us to actually run tests, which is current unsupported
  (https://github.com/doublify/pre-commit-rust/pull/19)

* Bump tempfile from 3.2.0 to 3.3.0. [dependabot[bot]]

  Bumps [tempfile](https://github.com/Stebalien/tempfile) from 3.2.0 to 3.3.0.
  - [Release notes](https://github.com/Stebalien/tempfile/releases)
  - [Changelog](https://github.com/Stebalien/tempfile/blob/master/NEWS)
  - [Commits](https://github.com/Stebalien/tempfile/compare/v3.2.0...v3.3.0)

  ---
  updated-dependencies:
  - dependency-name: tempfile
    dependency-type: direct:production
    update-type: version-update:semver-minor
  ...

## v22.1.0 (2022-01-02)

Happy new year! On this second day of 2022 comes a fresh release with one
notable new feature.

### New

* Support Obsidian's "Strict line breaks" setting. [Nick Groenen]

  This change introduces a new `--hard-linebreaks` CLI argument. When
  used, this converts soft line breaks to hard line breaks, mimicking
  Obsidian's "Strict line breaks" setting.

  > Implementation detail: I considered naming this flag
  > `--strict-line-breaks` to be consistent with Obsidian itself, however I
  > feel the name is somewhat misleading and ill-chosen.

### Other

* Give release binaries file extensions. [Nick Groenen]

  This may make it more clear to users that these are precompiled, binary
  files. This is especially relevant on Windows, where the convention is
  that executable files have a `.exe` extension, as seen in #49.

* Upgrade dependencies. [Nick Groenen]

  This commit upgrades all dependencies to their current latest versions. Most
  notably, this includes upgrades to the following most critical libraries:

      pulldown-cmark v0.8.0 -> v0.9.0
      pulldown-cmark-to-cmark v7.1.1 -> v9.0.0

  In total, these dependencies were upgraded:

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

* Bump serde_yaml from 0.8.21 to 0.8.23 (#52) [dependabot[bot]]

  Bumps [serde_yaml](https://github.com/dtolnay/serde-yaml) from 0.8.21 to 0.8.23.
  - [Release notes](https://github.com/dtolnay/serde-yaml/releases)
  - [Commits](https://github.com/dtolnay/serde-yaml/compare/0.8.21...0.8.23)

  ---
  updated-dependencies:
  - dependency-name: serde_yaml
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

* Bump pulldown-cmark-to-cmark from 7.1.0 to 7.1.1 (#51) [dependabot[bot]]

  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 7.1.0 to 7.1.1.
  - [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  - [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v7.1.0...v7.1.1)

  ---
  updated-dependencies:
  - dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

* Bump pulldown-cmark-to-cmark from 7.0.0 to 7.1.0 (#48) [dependabot[bot]]

  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 7.0.0 to 7.1.0.
  - [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  - [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v7.0.0...v7.1.0)

  ---
  updated-dependencies:
  - dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-minor
  ...

* Bump pulldown-cmark-to-cmark from 6.0.4 to 7.0.0 (#47) [dependabot[bot]]

  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 6.0.4 to 7.0.0.
  - [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  - [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v6.0.4...v7.0.0)

  ---
  updated-dependencies:
  - dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-major
  ...

* Bump pathdiff from 0.2.0 to 0.2.1 (#46) [dependabot[bot]]

  Bumps [pathdiff](https://github.com/Manishearth/pathdiff) from 0.2.0 to 0.2.1.
  - [Release notes](https://github.com/Manishearth/pathdiff/releases)
  - [Commits](https://github.com/Manishearth/pathdiff/commits)

  ---
  updated-dependencies:
  - dependency-name: pathdiff
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

* Bump pulldown-cmark-to-cmark from 6.0.3 to 6.0.4 (#44) [dependabot[bot]]

  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 6.0.3 to 6.0.4.
  - [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  - [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v6.0.3...v6.0.4)

  ---
  updated-dependencies:
  - dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

* Bump pretty_assertions from 0.7.2 to 1.0.0 (#45) [dependabot[bot]]

  Bumps [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) from 0.7.2 to 1.0.0.
  - [Release notes](https://github.com/colin-kiegel/rust-pretty-assertions/releases)
  - [Changelog](https://github.com/colin-kiegel/rust-pretty-assertions/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/colin-kiegel/rust-pretty-assertions/compare/v0.7.2...v1.0.0)

  ---
  updated-dependencies:
  - dependency-name: pretty_assertions
    dependency-type: direct:production
    update-type: version-update:semver-major
  ...

## v21.9.1 (2021-09-24)

### Changes

* Treat SVG files as embeddable images. [Narayan Sainaney]

  This will ensure SVG files are included as an image when using `![[foo.svg]]` syntax, as opposed to only being linked to.

### Other

* Bump pulldown-cmark-to-cmark from 6.0.2 to 6.0.3. [dependabot[bot]]

  Bumps [pulldown-cmark-to-cmark](https://github.com/Byron/pulldown-cmark-to-cmark) from 6.0.2 to 6.0.3.
  - [Release notes](https://github.com/Byron/pulldown-cmark-to-cmark/releases)
  - [Changelog](https://github.com/Byron/pulldown-cmark-to-cmark/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/Byron/pulldown-cmark-to-cmark/compare/v6.0.2...v6.0.3)

  ---
  updated-dependencies:
  - dependency-name: pulldown-cmark-to-cmark
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

* Bump serde_yaml from 0.8.20 to 0.8.21. [dependabot[bot]]

  Bumps [serde_yaml](https://github.com/dtolnay/serde-yaml) from 0.8.20 to 0.8.21.
  - [Release notes](https://github.com/dtolnay/serde-yaml/releases)
  - [Commits](https://github.com/dtolnay/serde-yaml/compare/0.8.20...0.8.21)

  ---
  updated-dependencies:
  - dependency-name: serde_yaml
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...



## v21.9.0 (2021-09-12)

> This release switches to a [calendar versioning scheme](https://calver.org/overview.html).
> Details on this decision can be read in [switching obsidian-export to CalVer](https://nick.groenen.me/posts/switching-obsidian-export-to-calver/).

### New

* Support postprocessors running on embedded notes. [Nick Groenen]

  This introduces support for postprocessors that are run on the result of
  a note that is being embedded into another note. This differs from the
  existing postprocessors (which remain unchanged) that run once all
  embeds have been processed and merged with the final note.

  These "embed postprocessors" may be set through the new
  `Exporter::add_embed_postprocessor` method.

* Add start_at option to export a partial vault. [Nick Groenen]

  This introduces a new `--start-at` CLI argument and corresponding
  `start_at()` method on the Exporter type that allows exporting of only a
  given subdirectory within a vault.

  See the updated README file for more details on when and how this may be
  used.

### Other

* Don't build docs for the bin target. [Nick Groenen]

  The library contains documentation covering both CLI and library usage,
  there's no separate documentation for just the binary target.

* Move postprocessor tests into their own file for clarity. [Nick Groenen]

* Update indirect dependencies. [Nick Groenen]

* Bump serde_yaml from 0.8.19 to 0.8.20. [dependabot[bot]]

  Bumps [serde_yaml](https://github.com/dtolnay/serde-yaml) from 0.8.19 to 0.8.20.
  - [Release notes](https://github.com/dtolnay/serde-yaml/releases)
  - [Commits](https://github.com/dtolnay/serde-yaml/compare/0.8.19...0.8.20)

  ---
  updated-dependencies:
  - dependency-name: serde_yaml
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

* Don't borrow references that are immediately dereferenced. [Nick Groenen]

  This was caught by a recently introduced clippy rule

* Bump serde_yaml from 0.8.17 to 0.8.19. [dependabot[bot]]

  Bumps [serde_yaml](https://github.com/dtolnay/serde-yaml) from 0.8.17 to 0.8.19.
  - [Release notes](https://github.com/dtolnay/serde-yaml/releases)
  - [Commits](https://github.com/dtolnay/serde-yaml/compare/0.8.17...0.8.19)

  ---
  updated-dependencies:
  - dependency-name: serde_yaml
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

* Update dependencies. [Nick Groenen]

* Fix 4 new clippy lints. [Nick Groenen]

* Bump regex from 1.4.6 to 1.5.3. [dependabot[bot]]

  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.6 to 1.5.3.
  - [Release notes](https://github.com/rust-lang/regex/releases)
  - [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  - [Commits](https://github.com/rust-lang/regex/compare/1.4.6...1.5.3)

* Bump pretty_assertions from 0.7.1 to 0.7.2. [dependabot[bot]]

  Bumps [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) from 0.7.1 to 0.7.2.
  - [Release notes](https://github.com/colin-kiegel/rust-pretty-assertions/releases)
  - [Changelog](https://github.com/colin-kiegel/rust-pretty-assertions/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/colin-kiegel/rust-pretty-assertions/compare/v0.7.1...v0.7.2)

* Bump regex from 1.4.5 to 1.4.6. [dependabot[bot]]

  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.5 to 1.4.6.
  - [Release notes](https://github.com/rust-lang/regex/releases)
  - [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  - [Commits](https://github.com/rust-lang/regex/compare/1.4.5...1.4.6)

## v0.7.0 (2021-04-11)

### New

* Postprocessing support. [Nick Groenen]

  Add support for postprocessing of Markdown prior to writing converted
  notes to disk.

  Postprocessors may be used when making use of Obsidian export as a Rust
  library to do the following:

  1. Modify a note's `Context`, for example to change the destination
     filename or update its Frontmatter.
  2. Change a note's contents by altering `MarkdownEvents`.
  3. Prevent later postprocessors from running or cause a note to be
     skipped entirely.

  Future releases of Obsidian export may come with built-in postprocessors
  for users of the command-line tool to use, if general use-cases can be
  identified.

  For example, a future release might include functionality to make notes
  more suitable for the Hugo static site generator. This functionality
  would be implemented as a postprocessor that could be enabled through
  command-line flags.

### Fixes

* Also percent-encode `?` in filenames. [Nick Groenen]

  A recent Obsidian update expanded the list of allowed characters in
  filenames, which now includes `?` as well. This needs to be
  percent-encoded for proper links in static site generators like Hugo.

### Other

* Bump pretty_assertions from 0.6.1 to 0.7.1. [dependabot[bot]]

  Bumps [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) from 0.6.1 to 0.7.1.
  - [Release notes](https://github.com/colin-kiegel/rust-pretty-assertions/releases)
  - [Changelog](https://github.com/colin-kiegel/rust-pretty-assertions/blob/main/CHANGELOG.md)
  - [Commits](https://github.com/colin-kiegel/rust-pretty-assertions/compare/v0.6.1...v0.7.1)

* Bump walkdir from 2.3.1 to 2.3.2. [dependabot[bot]]

  Bumps [walkdir](https://github.com/BurntSushi/walkdir) from 2.3.1 to 2.3.2.
  - [Release notes](https://github.com/BurntSushi/walkdir/releases)
  - [Commits](https://github.com/BurntSushi/walkdir/compare/2.3.1...2.3.2)

* Bump regex from 1.4.3 to 1.4.5. [dependabot[bot]]

  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.3 to 1.4.5.
  - [Release notes](https://github.com/rust-lang/regex/releases)
  - [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  - [Commits](https://github.com/rust-lang/regex/compare/1.4.3...1.4.5)

## v0.6.0 (2021-02-15)

### New

* Add `--version` flag. [Nick Groenen]

### Changes

* Don't Box FilterFn in WalkOptions. [Nick Groenen]

  Previously, `filter_fn` on the `WalkOptions` struct looked like:

      pub filter_fn: Option<Box<&'static FilterFn>>,

  This boxing was unneccesary and has been changed to:

      pub filter_fn: Option<&'static FilterFn>,

  This will only affect people who use obsidian-export as a library in
  other Rust programs, not users of the CLI.

  For those library users, they no longer need to supply `FilterFn`
  wrapped in a Box.

### Fixes

* Recognize notes beginning with underscores. [Nick Groenen]

  Notes with an underscore would fail to be recognized within Obsidian
  `[[_WikiLinks]]` due to the assumption that the underlying Markdown
  parser (pulldown_cmark) would emit the text between `[[` and `]]` as
  a single event.

  The note parser has now been rewritten to use a more reliable state
  machine which correctly recognizes this corner-case (and likely some
  others).

* Support self-references. [Joshua Coles]

  This ensures links to headings within the same note (`[[#Heading]]`)
  resolve correctly.

### Other

* Avoid redundant "Release" in GitHub release titles. [Nick Groenen]

* Add failing testcase for files with underscores. [Nick Groenen]

* Add unit tests for display of ObsidianNoteReference. [Nick Groenen]

* Add some unit tests for ObsidianNoteReference::from_str. [Nick Groenen]

* Also run tests on pull requests. [Nick Groenen]

* Apply clippy suggestions following rust 1.50.0. [Nick Groenen]

* Fix infinite recursion bug with references to current file. [Joshua Coles]

* Add tests for self-references. [Joshua Coles]

  Note as there is no support for block references at the moment, the generated link goes nowhere, however it is to a reasonable ID

* Bump tempfile from 3.1.0 to 3.2.0. [dependabot[bot]]

  Bumps [tempfile](https://github.com/Stebalien/tempfile) from 3.1.0 to 3.2.0.
  - [Release notes](https://github.com/Stebalien/tempfile/releases)
  - [Changelog](https://github.com/Stebalien/tempfile/blob/master/NEWS)
  - [Commits](https://github.com/Stebalien/tempfile/commits)

* Bump eyre from 0.6.3 to 0.6.5. [dependabot[bot]]

  Bumps [eyre](https://github.com/yaahc/eyre) from 0.6.3 to 0.6.5.
  - [Release notes](https://github.com/yaahc/eyre/releases)
  - [Changelog](https://github.com/yaahc/eyre/blob/v0.6.5/CHANGELOG.md)
  - [Commits](https://github.com/yaahc/eyre/compare/v0.6.3...v0.6.5)

* Bump regex from 1.4.2 to 1.4.3. [dependabot[bot]]

  Bumps [regex](https://github.com/rust-lang/regex) from 1.4.2 to 1.4.3.
  - [Release notes](https://github.com/rust-lang/regex/releases)
  - [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
  - [Commits](https://github.com/rust-lang/regex/compare/1.4.2...1.4.3)



## v0.5.1 (2021-01-10)

### Fixes

* Find uppercased notes when referenced with lowercase. [Nick Groenen]

  This commit fixes a bug where, if a note contained uppercase characters
  (for example `Note.md`) but was referred to using lowercase
  (`[[note]]`), that note would not be found.



## v0.5.0 (2021-01-05)

### New

* Add --no-recursive-embeds to break infinite recursion cycles. [Nick Groenen]

  It's possible to end up with "recursive embeds" when two notes embed
  each other. This happens for example when a `Note A.md` contains
  `![[Note B]]` but `Note B.md` also contains `![[Note A]]`.

  By default, this will trigger an error and display the chain of notes
  which caused the recursion.

  Using the new `--no-recursive-embeds`, if a note is encountered for a
  second time while processing the original note, rather than embedding it
  again a link to the note is inserted instead to break the cycle.

  See also: https://github.com/zoni/obsidian-export/issues/1

* Make walk options configurable on CLI. [Nick Groenen]

  By default hidden files, patterns listed in `.export-ignore` as well as
  any files ignored by git are excluded from exports. This behavior has
  been made configurable on the CLI using the new flags `--hidden`,
  `--ignore-file` and `--no-git`.

* Support links referencing headings. [Nick Groenen]

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

  [slug]: https://crates.io/crates/slug

* Support embeds referencing headings. [Nick Groenen]

  Previously, partial embeds (`![[note#heading]]`) would always include
  the entire file into the source note. Now, such embeds will only include
  the contents of the referenced heading (and any subheadings).

  Links and embeds of [arbitrary blocks] remains unsupported at this time.

  [arbitrary blocks]: https://publish.obsidian.md/help/How+to/Link+to+blocks

### Changes

* Print warnings to stderr rather than stdout. [Nick Groenen]

  Warning messages emitted when encountering broken links/references will
  now be printed to stderr as opposed to stdout.

### Other

* Include filter_fn field in WalkOptions debug display. [Nick Groenen]



## v0.4.0 (2020-12-23)

### Fixes

* Correct relative links within embedded notes. [Nick Groenen]

  Links within an embedded note would point to other local resources
  relative to the filesystem location of the note being embedded.

  When a note inside a different directory would embed such a note, these
  links would point to invalid locations.

  Now these links are calculated relative to the top note, which ensures
  these links will point to the right path.

### Other

* Add brief library documentation to all public types and functions. [Nick Groenen]



## v0.3.0 (2020-12-21)

### New

* Report file tree when RecursionLimitExceeded is hit. [Nick Groenen]

  This refactors the Context to maintain a list of all the files which
  have been processed so far in a chain of embeds. This information is
  then used to print a more helpful error message to users of the CLI when
  RecursionLimitExceeded is returned.

### Changes

* Add extra whitespace around multi-line warnings. [Nick Groenen]

  This makes errors a bit easier to distinguish after a number of warnings
  has been printed.

### Other

* Setup gitchangelog. [Nick Groenen]

  This adds a changelog (CHANGES.md) which is automatically generated with
  [gitchangelog].

  [gitchangelog]: https://github.com/vaab/gitchangelog



## v0.2.0 (2020-12-13)

* Allow custom filter function to be passed with WalkOptions. [Nick Groenen]

* Re-export vault_contents and WalkOptions as pub from crate root. [Nick Groenen]

* Run mdbook hook against README.md too. [Nick Groenen]

* Update installation instructions. [Nick Groenen]

  Installation no longer requires a git repository URL now that a crate is
  published.

* Add MdBook generation script and precommit hook. [Nick Groenen]

* Add more reliable non-ASCII tetscase. [Nick Groenen]

* Create FUNDING.yml. [Nick Groenen]

## v0.1.0 (2020-11-28)

* Public release. [Nick Groenen]
