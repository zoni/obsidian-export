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
