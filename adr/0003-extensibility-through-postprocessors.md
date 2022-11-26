# Extensibility through postprocessors

ADR #: 3 \
Date: 2021-02-20 \
Author: [Nick Groenen](https://github.com/zoni/)

## Context

It's desirable for end-users to have some control over the logic that is used to export notes and the transformation of their content from Obsidian-flavored markdown to regular markdown.

One use-case would be to tailor the output for consumption by a specific static site generator, for example [Hugo].
This requires emitting specific frontmatter elements and converting certain syntax elements to Hugo [shortcodes].

However, to ease maintenance the core of the library would ideally remain as narrowly scoped and limited as possible.
Ideally, all of such customization would be expressed through some kind of hook, callback or plugin mechanism that keeps it entirely out of the core of the obsidian-export library modules.

## Decision

We introduce the concept of _postprocessors_, which are (user-supplied) Rust functions that are called for every exported note right after it's been parsed, but before it is written out to the filesystem.

Postprocessors may be chained (they'll be called in order, with the output of the first being the input to the second, etc) and will have access to and be able to modify:

1. The stream of markdown events which makes up the note
2. The note context, containing information such as the filename, path, frontmatter, etc.

In addition, the return value of a postprocessor will be used to affect how the note is treated further, to prevent later postprocessors from running (`PostprocessorResult::StopHere`) or cause a note to be skipped entirely (`PostprocessorResult::StopAndSkipNote`) and omitted from the export.

In code, the function signature for a postprocessor looks like:

```rust
pub type Postprocessor = dyn Fn(Context, MarkdownEvents) -> (Context, MarkdownEvents, PostprocessorResult) + Send + Sync;
```

The `Exporter` will receive a new method `add_postprocessor()` to allow users to register their desired postprocessors.

Initially, we'll introduce support for this without anything else, but if any sufficiently generic usecases can be identified, we may add certain postprocessors to obsidian-export directly for users to opt-in to via CLI args.

[Hugo]: https://gohugo.io/
[shortcodes]: https://gohugo.io/content-management/shortcodes/
