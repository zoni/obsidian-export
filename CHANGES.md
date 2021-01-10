# Changelog

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
