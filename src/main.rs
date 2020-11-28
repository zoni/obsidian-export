use eyre::{eyre, Result};
use gumdrop::Options;
use obsidian_export::{Exporter, FrontmatterStrategy};
use std::path::PathBuf;

#[derive(Debug, Options)]
struct Opts {
    #[options(help = "Display program help")]
    help: bool,

    #[options(help = "Source file containing reference", free, required)]
    source: Option<PathBuf>,

    #[options(help = "Destination file being linked to", free, required)]
    destination: Option<PathBuf>,

    #[options(
        help = "Frontmatter strategy (one of: always, never, auto)",
        no_short,
        long = "frontmatter",
        parse(try_from_str = "frontmatter_strategy_from_str"),
        default = "auto"
    )]
    frontmatter_strategy: FrontmatterStrategy,
}

fn frontmatter_strategy_from_str(input: &str) -> Result<FrontmatterStrategy> {
    match input {
        "auto" => Ok(FrontmatterStrategy::Auto),
        "always" => Ok(FrontmatterStrategy::Always),
        "never" => Ok(FrontmatterStrategy::Never),
        _ => Err(eyre!("must be one of: always, never, auto")),
    }
}

fn main() -> Result<()> {
    let args = Opts::parse_args_default_or_exit();
    let source = args.source.unwrap();
    let destination = args.destination.unwrap();

    let mut exporter = Exporter::new(source, destination);
    exporter.frontmatter_strategy(args.frontmatter_strategy);
    // TODO: Pass in configurable walk_options here: exporter.walk_options(..);
    // TODO: This should allow settings for ignore_hidden and honor_gitignore.
    exporter.run()?;

    Ok(())
}
