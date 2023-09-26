# Contributing to Obsidian Export

Hi there!
Thank you so much for wanting to contribute to this project.
I greatly appreciate any efforts people like you put into making obsidian-export better!

Managing an open-source project can take a lot of time and effort however.
As this is a passion project which I maintain alongside my regular daytime job, I need to take some measures to safeguard my mental health and the enjoyment of this project.

This document aims to provide guidance which makes contributions easier by:

1. Defining the expectations I have of submissions to the codebase and the pull request process.
2. Helping you get set up for development on the code.
3. Providing pointers to some areas of the codebase, as well as some design considerations to take into account when making changes.

## Working with Rust

Obsidian-export is written in [Rust](https://www.rust-lang.org/), which is not the easiest of languages to master.
If you'd like to contribute but you don't know Rust, check out [Learn Rust](https://www.rust-lang.org/learn) for some suggestions of how to get started with the language.
In general, I will do my best to support you and help you out, but understand my time for mentoring is highly limited.

To work on the codebase, you'll also need the Rust toolchain, including cargo, rustfmt and clippy.
The easiest way is to [install Rust using rustup](https://www.rust-lang.org/tools/install), which lets you install rustfmt and clippy using `rustup component add rustfmt` and `rustup component add clippy` respectively.

## Design principles

My intention is to keep the core of `obsidian-export` as limited and small as possible, avoiding changes to the core [`Exporter`](https://docs.rs/obsidian-export/latest/obsidian_export/struct.Exporter.html) struct or any of its methods whenever possible.
This improves long-term maintainability and makes investigation of bugs simpler.

To keep the core of obsidian-export small while still supporting a wide range of use-cases, additional functionality should be pushed down into [postprocessors](https://docs.rs/obsidian-export/latest/obsidian_export/type.Postprocessor.html) as much as possible.
You can see some examples of this in:

- [Support Obsidian's "Strict line breaks" setting (#57)](https://github.com/zoni/obsidian-export/pull/57)
- [Frontmatter based filtering (#67)](https://github.com/zoni/obsidian-export/pull/67)

## Conventions

Code is formatted with [rustfmt](https://github.com/rust-lang/rustfmt) using the default options.
In addition, all default [clippy](https://github.com/rust-lang/rust-clippy) checks on the latest stable Rust compiler must also pass.
Both of these are enforced through CI using GitHub actions.

> **💡 Tip: install pre-commit hooks**
>
> This codebase is set up with the [pre-commit framework](https://pre-commit.com/) to automatically run the appropriate checks locally whenever you commit.
> Assuming you [have pre-commit installed](https://pre-commit.com/#install), all you need to do is run `pre-commit install` once to get this set up.

Following my advice on [creating high-quality commits](https://nick.groenen.me/notes/high-quality-commits/) will make it easier for me to review changes.
I don't insist on this, but pull requests which fail to adhere to these conventions are at risk of being squashed and having their commit messages rewritten when they are accepted.

## Tests

In order to have confidence that your changes work as intended, as well as to avoid regressions when making changes in the future, I would like to see code accompanied by test cases.

At the moment, the test framework primary relies on high-level integration tests, all of which are defined in the [tests](tests/) directory.
These rely on comparing Markdown notes [before](tests/testdata/input) and [after](tests/testdata/expected) running an export.
By studying some of the existing tests, you should be able to copy and adapt these for your own changes.

For an example of doing low-level unit tests, you can look at the end of [frontmatter.rs](src/frontmatter.rs).

## Documentation

I place a lot of value on good documentation and would encourage you to include updates to the docs with your changes.
Changes or additions to public methods and attributes **must** come with proper documentation for a PR to be accepted.

Advice on writing Rust documentation can be found in:

- [The rustdoc book: How to write documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [Rust by example: Documentation](https://doc.rust-lang.org/rust-by-example/meta/doc.html)

Updates to the user guide/README instructions are also preferred, but optional.
If you don't feel comfortable writing user documentation, I will be happy to guide you or do it for you.

> **⚠ Warning**
>
> If you update the README file, take note that you must edit the fragments in the [docs](docs/) directory as opposed to the README in the root of the repository, which is auto-generated.
