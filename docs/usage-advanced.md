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

```
# Ignore the directory private that is located at the top of the export tree
/private
# Ignore any file or directory called `test`
test
# Ignore any PDF file
*.pdf
# ..but include special.pdf
!special.pdf
```

For more comprehensive documentation and examples, see the [gitignore] manpage.

## Recursive embeds

It's possible to end up with "recursive embeds" when two notes embed each other.
This happens for example when a `Note A.md` contains `![[Note B]]` but `Note B.md` also contains `![[Note A]]`.

By default, this will trigger an error and display the chain of notes which caused the recursion.

This behavior may be changed by specifying `--no-recursive-embeds`.
Using this mode, if a note is encountered for a second time while processing the original note, instead of embedding it again a link to the note is inserted instead to break the cycle.

[gitignore]: https://git-scm.com/docs/gitignore
