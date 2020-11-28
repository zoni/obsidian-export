## Installation

I don't currently provide binary releases, though I may create these if there is sufficient demand.
Until then, users can install *obsidian-export* from source using [Cargo](https://doc.rust-lang.org/cargo/):

1. Install the Rust toolchain: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
1. Run: `cargo install https://github.com/zoni/obsidian-export.git --locked`

The same `cargo install` command can be used to upgrade to a newer version as well.

### Supported Operating Systems

Obsidian-export has only been tested on GNU/Linux, but should run on any modern Unix-like system.

Windows has not been tested and is unsupported at this time.
Experience reports from Windows users would be welcome however, and Windows support may be considered if the current UTF-8 filename assumption (see below) can hold true on Windows.

### Character encodings

At present, UTF-8 character encoding is assumed for all note text as well as filenames.
All text and file handling performs [lossy conversion to Unicode strings](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy).

Use of non-UTF8 encodings may lead to issues like incorrect text replacement and failure to find linked notes.
While this may change in the future, there are no plans to change this behavior on the short term.
