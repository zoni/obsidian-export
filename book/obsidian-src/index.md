# Obsidian Export

_Rust library and associated CLI program to export an [Obsidian] vault to regular Markdown (specifically: [CommonMark])_

- Recursively export Obsidian Markdown files to CommonMark.
- Supports `[[note]]`-style references as well as `![[note]]` file includes.
    - `[[note#heading]]` linking/embedding not yet supported, but planned.
- Support for [gitignore]-style exclude patterns (default: `.export-ignore`).
- Automatically excludes files that are ignored by Git when the vault is located in a Git repository.

Please note obsidian-export is not officially endorsed by the Obsidian team.
It supports most but not all of Obsidian's Markdown flavor.

[Obsidian]: https://obsidian.md/
[CommonMark]: https://commonmark.org/
[gitignore]: https://git-scm.com/docs/gitignore
