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

Pre-compiled binaries for all major platforms are available at <https://github.com/zoni/obsidian-export/releases>

In addition to the installation scripts provided, these releases are also suitable for [installation with cargo-binstall](https://github.com/cargo-bins/cargo-binstall#readme).

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

The following files are not exported by default:

* hidden files (can be adjusted with `--hidden`)
* files matching a pattern listed in `.export-ignore` (can be adjusted with `--ignore-file`)
* any files that are ignored by git (can be adjusted with `--no-git`)
* using `--skip-tags foo --skip-tags bar` will skip any files that have the tags `foo` or `bar` in their frontmatter
* using `--only-tags foo --only-tags bar` will skip any files that **don't** have the tags `foo` or `bar` in their frontmatter

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


# Contributing

I will happily accept bug fixes as well as enhancements, as long as they align with the overall scope and vision of the project.
Please see [CONTRIBUTING](CONTRIBUTING.md) for more information.


# License

Obsidian-export is open-source software released under the [BSD-2-Clause Plus Patent License].
This license is designed to provide: a) a simple permissive license; b) that is compatible with the GNU General Public License (GPL), version 2; and c) which also has an express patent grant included.

Please review the [LICENSE] file for the full text of the license.


# Changelog

For a list of releases and the changes with each version, please refer to the [CHANGELOG](CHANGELOG.md).

[Obsidian]: https://obsidian.md/
[CommonMark]: https://commonmark.org/
[gitignore]: https://git-scm.com/docs/gitignore
[Cargo]: https://doc.rust-lang.org/cargo/
[Hugo]: https://gohugo.io
[`ref` and `relref` shortcodes]: https://gohugo.io/content-management/cross-references/
[Markdown Render Hooks]: https://gohugo.io/getting-started/configuration-markup#markdown-render-hooks
[BSD-2-Clause Plus Patent License]: https://spdx.org/licenses/BSD-2-Clause-Patent.html
[LICENSE]: LICENSE
