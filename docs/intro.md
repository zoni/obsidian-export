# Obsidian Export

_Obsidian Export is a CLI program and a Rust library to export an [Obsidian] vault to regular Markdown._

- Recursively export Obsidian Markdown files to [CommonMark].
- Supports `[[note]]`-style references as well as `![[note]]` file includes.
- Support for [gitignore]-style exclude patterns (default: `.export-ignore`).
- Automatically excludes files that are ignored by Git when the vault is located in a Git repository.
- Runs on all major platforms: Windows, Mac, Linux, BSDs.

Please note obsidian-export is not officially endorsed by the Obsidian team.
It supports most but not all of Obsidian's Markdown flavor.

[Obsidian]: https://obsidian.md/
[CommonMark]: https://commonmark.org/
[gitignore]: https://git-scm.com/docs/gitignore
