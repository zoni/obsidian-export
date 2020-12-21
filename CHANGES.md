# Changelog

## (unreleased)

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
