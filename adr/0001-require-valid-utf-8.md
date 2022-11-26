# Require valid UTF-8

ADR #: 1 \
Date: 2020-11-28 \
Author: [Nick Groenen](https://github.com/zoni/)

## Context

Rust's native [String] types are UTF-8–encoded (an [OsString] can hold arbitrary byte sequences), but filesystem paths (represented by the [Path] and [PathBuf]) structs) may consist of arbitrary encodings/byte sequences.
Similarly, note content that we read from files could be encoded in any arbitrary encoding; it may not consist of valid UTF-8.

In many cases we will need to look up strings found within notes against a list of paths (for example to find the path in the vault when encountering a `[[WikiLinkedNote]]`).

We must decide whether to treat everything as valid UTF-8, or to treat it as arbitrary bytes, as we cannot mix these two together.

## Decision

Treating everything as arbitrary byte slices is technically the more correct thing to do, but it would complicate the internal design and is more difficult to get right.
We can then no longer trivially perform certain operations like upper/lowercasing, splitting/appending, etc. as doing so might lead to mixed encoding schemes.

To simplify the code and eliminate many sources of edge-cases introduced by possible mixed encoding schemes, we will shift the responsibility to end-users to ensure all input to obsidian-export is valid UTF-8.

Where applicable, we will use lossy conversion functions such as `to_string_lossy()` and `from_utf8_lossy()` to simplify code by not having to handle the error-case of attempting to convert bytes that are not valid UTF-8.

[String]: https://doc.rust-lang.org/std/string/struct.String.html
[OsString]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
[Path]: https://doc.rust-lang.org/std/path/struct.Path.html
[PathBuf]: https://doc.rust-lang.org/std/path/struct.PathBuf.html
