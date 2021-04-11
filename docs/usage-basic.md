# Basic usage

The main interface of _obsidian-export_ is the `obsidian-export` CLI command.
As a text interface, this must be run from a terminal or Windows PowerShell.

It is assumed that you have basic familiarity with command-line interfaces and that you set up your `PATH` correctly if you installed with `cargo`. 
Running `obsidian-export --version` should print a version number rather than giving some kind of error.

> If you downloaded a pre-built binary and didn't put it a location referenced by `PATH` (for example, you put it in `Downloads`), you will need to provide the full path to the binary instead.
>
> For example `~/Downloads/obsidian-export --version` on Mac/Linux or `~\Downloads\obsidian-export --version` on Windows (PowerShell).

In it's most basic form, `obsidian-export` takes just two mandatory arguments, a source and a destination:

```sh
obsidian-export /path/to/my-obsidian-vault /path/to/exported-notes/
```

This will export all of the files from `my-obsidian-vault` to `exported-notes`, except for those listed in `.export-ignore` or `.gitignore`.

> Note that the destination directory must exist, so you may need to create a new, empty directory first.
>
> If you give it an **existing** directory, files under that directory may get overwritten.

It is also possible to export individual files:

```sh
# Export as some-note.md to /tmp/export/
obsidian-export my-obsidian-vault/some-note.md /tmp/export/
# Export as exported-note.md in /tmp/
obsidian-export my-obsidian-vault/some-note.md /tmp/exported-note.md
```

## Character encodings

At present, UTF-8 character encoding is assumed for all note text as well as filenames.
All text and file handling performs [lossy conversion to Unicode strings][from_utf8_lossy].

Use of non-UTF8 encodings may lead to issues like incorrect text replacement and failure to find linked notes.
While this may change in the future, there are no plans to change this behavior in the short term.

[from_utf8_lossy]: https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy
