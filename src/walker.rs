use std::fmt;
use std::path::{Path, PathBuf};

use ignore::{DirEntry, Walk, WalkBuilder};
use snafu::ResultExt;

use crate::{ExportError, WalkDirSnafu};

type Result<T, E = ExportError> = std::result::Result<T, E>;
type FilterFn = dyn Fn(&DirEntry) -> bool + Send + Sync + 'static;

/// `WalkOptions` specifies how an Obsidian vault directory is scanned for eligible files to export.
#[derive(Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct WalkOptions<'a> {
    /// The filename for ignore files, following the
    /// [gitignore](https://git-scm.com/docs/gitignore) syntax.
    ///
    /// By default `.export-ignore` is used.
    pub ignore_filename: &'a str,
    /// Whether to ignore hidden files.
    ///
    /// This is enabled by default.
    pub ignore_hidden: bool,
    /// Whether to honor git's ignore rules (`.gitignore` files, `.git/config/exclude`, etc) if
    /// the target is within a git repository.
    ///
    /// This is enabled by default.
    pub honor_gitignore: bool,
    /// An optional custom filter function which is called for each directory entry to determine if
    /// it should be included or not.
    ///
    /// This is passed to [`ignore::WalkBuilder::filter_entry`].
    pub filter_fn: Option<&'static FilterFn>,
}

impl<'a> fmt::Debug for WalkOptions<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filter_fn_fmt = match self.filter_fn {
            Some(_) => "<function set>",
            None => "<not set>",
        };
        f.debug_struct("WalkOptions")
            .field("ignore_filename", &self.ignore_filename)
            .field("ignore_hidden", &self.ignore_hidden)
            .field("honor_gitignore", &self.honor_gitignore)
            .field("filter_fn", &filter_fn_fmt)
            .finish()
    }
}

impl<'a> WalkOptions<'a> {
    /// Create a new set of options using default values.
    #[must_use]
    pub fn new() -> Self {
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

/// `vault_contents` returns all of the files in an Obsidian vault located at `path` which would be
/// exported when using the given [`WalkOptions`].
pub fn vault_contents(root: &Path, opts: WalkOptions<'_>) -> Result<Vec<PathBuf>> {
    let mut contents = Vec::new();
    let walker = opts.build_walker(root);
    for entry in walker {
        let entry = entry.context(WalkDirSnafu { path: root })?;
        let path = entry.path();
        let metadata = entry.metadata().context(WalkDirSnafu { path })?;

        if metadata.is_dir() {
            continue;
        }
        contents.push(path.to_path_buf());
    }
    Ok(contents)
}
