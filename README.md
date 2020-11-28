# Obsidian Export

*Rust library and associated CLI program to export an [Obsidian](https://obsidian.md/) vault to regular Markdown (specifically: [CommonMark](https://commonmark.org/))*

* Recursively export Obsidian Markdown files to CommonMark.
* Supports `[[note]]`-style references as well as `![[note]]` file includes.
  * `[[note#heading]]` linking/embedding not yet supported, but planned.
* Support for [gitignore](https://git-scm.com/docs/gitignore)-style exclude patterns (default: `.export-ignore`).
* Automatically excludes files that are ignored by Git when the vault is located in a Git repository.

Please note obsidian-export is not officially endorsed by the Obsidian team.
It supports most but not all of Obsidian's Markdown flavor.


## Installation

 > 
 > **Note**: 
 > *Obsidian-export* has been developed on Linux.
 > Windows and Mac OS are covered as part of the continuous integration tests run on GitHub, but these have not been tested by the author.
 > Experience reports from users on these operating systems would be welcomed.

Binary releases for x86-64 processors are provided for Windows, Linux and Mac operating systems on a best-effort basis.
These may be downloaded from: [https://github.com/zoni/obsidian-export/releases](https://github.com/zoni/obsidian-export/releases)

Alternatively, *obsidian-export* may be compiled from source using [Cargo](https://doc.rust-lang.org/cargo/), the official package manager for Rust, by using the following steps:

1. Install the Rust toolchain: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
1. Run: `cargo install https://github.com/zoni/obsidian-export.git --locked`

The same `cargo install` command can later be used to upgrade to a newer release as well.


## Usage

The main interface of *obsidian-export* is the `obsidian-export` CLI command.
In it's most basic form, `obsidian-export` takes just two mandatory arguments, a source and a destination:

````sh
obsidian-export my-obsidian-vault /tmp/export
````

This will export all of the files from `my-obsidian-vault` to `/tmp/export`, except for those listed in `.export-ignore` or `.gitignore`.

It is also possible to export individual files:

````sh
# Export as some-note.md to /tmp/export/
obsidian-export my-obsidian-vault/some-note.md /tmp/export/
# Export as exported-note.md in /tmp/
obsidian-export my-obsidian-vault/some-note.md /tmp/exported-note.md
````

### Character encodings

At present, UTF-8 character encoding is assumed for all note text as well as filenames.
All text and file handling performs [lossy conversion to Unicode strings](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy).

Use of non-UTF8 encodings may lead to issues like incorrect text replacement and failure to find linked notes.
While this may change in the future, there are no plans to change this behavior in the short term.

### Frontmatter

By default, frontmatter is copied over "as-is".

Some static site generators are picky about frontmatter and require it to be present.
Some get tripped up when Markdown files don't have frontmatter but start with a list item or horizontal rule.
In these cases, `--frontmatter=always` can be used to insert an empty frontmatter entry.

To completely remove any frontmatter from exported notes, use `--frontmatter=never`.

### Ignoring files

By default, hidden files, patterns listed in `.export-ignore` as well as any files ignored by git (if your vault is part of a git repository) will be excluded from exports.
These options will become configurable in the next release.

Notes linking to ignored notes will be unlinked (they'll only include the link text). Embeds of ignored notes will be skipped entirely.

#### Ignorefile syntax

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


## License

Obsidian-export is dual-licensed under the [Apache 2.0](https://github.com/zoni/obsidian-export/blob/master/LICENSE-APACHE) and the [MIT](https://github.com/zoni/obsidian-export/blob/master/LICENSE-MIT) licenses.
