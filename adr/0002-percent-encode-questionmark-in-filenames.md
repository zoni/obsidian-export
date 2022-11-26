# Percent-encode `?` in filenames

ADR #: 2 \
Date: 2021-02-16 \
Author: [Nick Groenen](https://github.com/zoni/)

## Context

A recent Obsidian update expanded the list of allowed characters in filenames, which now includes `?` as well.
Most static site generators break when they encounter a bare `?` in markdown links, so this should be percent-encoded to ensure we export valid links.

## Decision

We'll add `?` to the hardcoded list of characters to escape (`const PERCENTENCODE_CHARS`).
Making this list configurable is desirable, but this is left for a future improvement given other priorities.
