# Installation

## Pre-built binaries

Binary releases for x86-64 processors are provided for Windows, Linux and Mac operating systems on a best-effort basis.
They are built with GitHub runners as part of the release workflow defined in `.github/workflows/release.yml`.

The resulting binaries can be downloaded from <https://github.com/zoni/obsidian-export/releases>

## Building from source

When binary releases are unavailable for your platform, or you do not trust the pre-built binaries, then _obsidian-export_ can be compiled from source with relatively little effort.
This is done through [Cargo], the official package manager for Rust, with the following steps:

1. Install the Rust toolchain from <https://www.rust-lang.org/tools/install>
2. Run: `cargo install obsidian-export`

> It is expected that you successfully configured the PATH variable correctly while installing the Rust toolchain, as described under _"Configuring the PATH environment variable"_ on <https://www.rust-lang.org/tools/install>.

## Upgrading from earlier versions

If you downloaded a pre-built binary, upgrade by downloading the latest version to replace the old one.

If you built from source, upgrade by running `cargo install obsidian-export` again.

[Cargo]: https://doc.rust-lang.org/cargo/
