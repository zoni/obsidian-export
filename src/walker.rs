use crate::{ExportError, WalkDirError};
use ignore::{DirEntry, Walk, WalkBuilder};
use snafu::ResultExt;
use std::fmt;
use std::path::{Path, PathBuf};

type Result<T, E = ExportError> = std::result::Result<T, E>;
type FilterFn = dyn Fn(&DirEntry) -> bool + Send + Sync + 'static;

#[derive(Clone)]
pub struct WalkOptions<'a> {
    pub ignore_filename: &'a str,
    pub ignore_hidden: bool,
    pub honor_gitignore: bool,
    pub filter_fn: Option<Box<&'static FilterFn>>,
}

impl<'a> fmt::Debug for WalkOptions<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WalkOptions")
            .field("ignore_filename", &self.ignore_filename)
            .field("ignore_hidden", &self.ignore_hidden)
            .field("honor_gitignore", &self.honor_gitignore)
            .finish()
    }
}

impl<'a> WalkOptions<'a> {
    pub fn new() -> WalkOptions<'a> {
        WalkOptions {
            ignore_filename: ".export-ignore",
            ignore_hidden: true,
            honor_gitignore: true,
            filter_fn: None,
        }
    }

    fn build_walker(self, path: &Path) -> Walk {
        let mut walker = WalkBuilder::new(path);
        walker
            .standard_filters(false)
            .parents(true)
            .hidden(self.ignore_hidden)
            .add_custom_ignore_filename(self.ignore_filename)
            .require_git(true)
            .git_ignore(self.honor_gitignore)
            .git_global(self.honor_gitignore)
            .git_exclude(self.honor_gitignore);

        if let Some(filter) = self.filter_fn {
            walker.filter_entry(filter);
        }
        walker.build()
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
