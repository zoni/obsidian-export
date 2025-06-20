use std::path::{Path, PathBuf};

use crate::Frontmatter;

#[derive(Debug, Clone)]
/// Context holds metadata about a note which is being parsed.
///
/// This is used internally to keep track of nesting and help with constructing proper references
/// to other notes.
///
/// It is also passed to [postprocessors][crate::Postprocessor] to provide contextual information
/// and allow modification of a note's frontmatter.
pub struct Context {
    file_tree: Vec<PathBuf>,

    /// The path where this note will be written to when exported.
    ///
    /// Changing this path will result in the note being written to that new path instead, but
    /// beware: links will not be updated automatically.  If this is changed by a
    /// [postprocessor][crate::Postprocessor], it's up to that postprocessor to rewrite any
    /// existing links to this new path.
    pub destination: PathBuf,

    /// The [Frontmatter] for this note. Frontmatter may be modified in-place (see
    /// [`serde_yaml::Mapping`] for available methods) or replaced entirely.
    ///
    /// # Example
    ///
    /// Insert `foo: bar` into a note's frontmatter:
    ///
    /// ```
    /// # use obsidian_export::Frontmatter;
    /// # use obsidian_export::Context;
    /// # use std::path::PathBuf;
    /// use obsidian_export::serde_yaml::Value;
    ///
    /// # let mut context = Context::new(PathBuf::from("source"), PathBuf::from("destination"));
    /// let key = Value::String("foo".to_string());
    ///
    /// context
    ///     .frontmatter
    ///     .insert(key.clone(), Value::String("bar".to_string()));
    /// ```
    pub frontmatter: Frontmatter,
}

impl Context {
    /// Create a new `Context`
    #[inline]
    #[must_use]
    pub fn new(src: PathBuf, dest: PathBuf) -> Self {
        Self {
            file_tree: vec![src],
            destination: dest,
            frontmatter: Frontmatter::new(),
        }
    }

    /// Create a new `Context` which inherits from a parent Context.
    #[inline]
    #[must_use]
    pub fn from_parent(context: &Self, child: &Path) -> Self {
        let mut context = context.clone();
        context.file_tree.push(child.to_path_buf());
        context
    }

    /// Return the path of the file currently being parsed.
    #[inline]
    #[must_use]
    pub fn current_file(&self) -> &PathBuf {
        self.file_tree
            .last()
            .expect("Context not initialized properly, file_tree is empty")
    }

    /// Return the path of the root file.
    ///
    /// Typically this will yield the same element as `current_file`, but when a note is embedded
    /// within another note, this will return the outer-most note.
    #[inline]
    #[must_use]
    pub fn root_file(&self) -> &PathBuf {
        self.file_tree
            .first()
            .expect("Context not initialized properly, file_tree is empty")
    }

    /// Return the note depth (nesting level) for this context.
    #[inline]
    #[must_use]
    pub const fn note_depth(&self) -> usize {
        self.file_tree.len()
    }

    /// Return the list of files associated with this context.
    ///
    /// The first element corresponds to the root file, the final element corresponds to the file
    /// which is currently being processed (see also `current_file`).
    #[inline]
    #[must_use]
    pub fn file_tree(&self) -> Vec<PathBuf> {
        self.file_tree.clone()
    }
}
