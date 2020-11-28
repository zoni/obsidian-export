use crate::{ExportError, WalkDirError};
use ignore::{Walk, WalkBuilder};
use snafu::ResultExt;
use std::path::{Path, PathBuf};

type Result<T, E = ExportError> = std::result::Result<T, E>;

#[derive(Debug, Clone, Copy)]
pub struct WalkOptions<'a> {
    ignore_filename: &'a str,
    ignore_hidden: bool,
    honor_gitignore: bool,
}

impl<'a> WalkOptions<'a> {
    pub fn new() -> WalkOptions<'a> {
        WalkOptions {
            ignore_filename: ".export-ignore",
            ignore_hidden: true,
            honor_gitignore: true,
        }
    }

    fn build_walker(self, path: &Path) -> Walk {
        WalkBuilder::new(path)
            .standard_filters(false)
            .parents(true)
            .hidden(self.ignore_hidden)
            .add_custom_ignore_filename(self.ignore_filename)
            .require_git(true)
            .git_ignore(self.honor_gitignore)
            .git_global(self.honor_gitignore)
            .git_exclude(self.honor_gitignore)
            .build()
    }
}

impl<'a> Default for WalkOptions<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn vault_contents(path: &Path, opts: WalkOptions) -> Result<Vec<PathBuf>> {
    let mut contents = Vec::new();
    let walker = opts.build_walker(path);
    for entry in walker {
        let entry = entry.context(WalkDirError { path })?;
        let path = entry.path();
        let metadata = entry.metadata().context(WalkDirError { path })?;

        if metadata.is_dir() {
            continue;
        }
        contents.push(path.to_path_buf());
    }
    Ok(contents)
}
