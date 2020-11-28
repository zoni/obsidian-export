## Usage

The main interface of obsidian-export is the similarly-named `obsidian-export` CLI command.
In it's most basic form, `obsidian-export` takes just two mandatory arguments, source and destination:

obsidian-export ~/Knowledgebase /tmp/export

This will export all of the files from `~/Knowledgebase` to `/tmp/export`, except for those listed in `.export-ignore` or `.gitignore`.

### Frontmatter

By default, frontmatter is copied over "as-is".

Some static site generators are picky about frontmatter and require it to be present.
Some get tripped up when Markdown files don't have frontmatter but start with a list item or horizontal rule.
In these cases, `--frontmatter=always` can be used to insert an empty frontmatter entry.

To completely remove any frontmatter from exported notes, use `--frontmatter=never`.
