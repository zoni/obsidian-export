pub extern crate pulldown_cmark;
pub extern crate serde_yaml;

#[macro_use]
extern crate lazy_static;

mod context;
mod frontmatter;
pub mod postprocessors;
mod references;
mod walker;

pub use context::Context;
pub use frontmatter::{Frontmatter, FrontmatterStrategy};
pub use walker::{vault_contents, WalkOptions};

use frontmatter::{frontmatter_from_str, frontmatter_to_str};
use pathdiff::diff_paths;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark_with_options;
use rayon::prelude::*;
use references::*;
use slug::slugify;
use snafu::{ResultExt, Snafu};
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::str;

/// A series of markdown [Event]s that are generated while traversing an Obsidian markdown note.
pub type MarkdownEvents<'a> = Vec<Event<'a>>;

/// A post-processing function that is to be called after an Obsidian note has been fully parsed and
/// converted to regular markdown syntax.
///
/// Postprocessors are called in the order they've been added through [Exporter::add_postprocessor]
/// just before notes are written out to their final destination.
/// They may be used to achieve the following:
///
/// 1. Modify a note's [Context], for example to change the destination filename or update its [Frontmatter] (see [Context::frontmatter]).
/// 2. Change a note's contents by altering [MarkdownEvents].
/// 3. Prevent later postprocessors from running ([PostprocessorResult::StopHere]) or cause a note
///    to be skipped entirely ([PostprocessorResult::StopAndSkipNote]).
///
/// # Postprocessors and embeds
///
/// Postprocessors normally run at the end of the export phase, once notes have been fully parsed.
/// This means that any embedded notes have been resolved and merged into the final note already.
///
/// In some cases it may be desirable to change the contents of these embedded notes *before* they
/// are inserted into the final document. This is possible through the use of
/// [Exporter::add_embed_postprocessor].
/// These "embed postprocessors" run much the same way as regular postprocessors, but they're run on
/// the note that is about to be embedded in another note. In addition:
///
/// - Changes to context carry over to later embed postprocessors, but are then discarded. This
///   means that changes to frontmatter do not propagate to the root note for example.
/// - [PostprocessorResult::StopAndSkipNote] prevents the embedded note from being included (it's
///   replaced with a blank document) but doesn't affect the root note.
///
/// It's possible to pass the same functions to [Exporter::add_postprocessor] and
/// [Exporter::add_embed_postprocessor]. The [Context::note_depth] method may be used to determine
/// whether a note is a root note or an embedded note in this situation.
///
/// # Examples
///
/// ## Update frontmatter
///
/// This example shows how to make changes a note's frontmatter. In this case, the postprocessor is
/// defined inline as a closure.
///
/// ```
/// use obsidian_export::serde_yaml::Value;
/// use obsidian_export::{Exporter, PostprocessorResult};
/// # use std::path::PathBuf;
/// # use tempfile::TempDir;
///
/// # let tmp_dir = TempDir::new().expect("failed to make tempdir");
/// # let source = PathBuf::from("tests/testdata/input/postprocessors");
/// # let destination = tmp_dir.path().to_path_buf();
/// let mut exporter = Exporter::new(source, destination);
///
/// // add_postprocessor registers a new postprocessor. In this example we use a closure.
/// exporter.add_postprocessor(&|context, _events| {
///     // This is the key we'll insert into the frontmatter. In this case, the string "foo".
///     let key = Value::String("foo".to_string());
///     // This is the value we'll insert into the frontmatter. In this case, the string "bar".
///     let value = Value::String("baz".to_string());
///
///     // Frontmatter can be updated in-place, so we can call insert on it directly.
///     context.frontmatter.insert(key, value);
///
///     // This return value indicates processing should continue.
///     PostprocessorResult::Continue
/// });
///
/// exporter.run().unwrap();
/// ```
///
/// ## Change note contents
///
/// In this example a note's markdown content is changed by iterating over the [MarkdownEvents] and
/// changing the text when we encounter a [text element][Event::Text].
///
/// Instead of using a closure like above, this example shows how to use a separate function
/// definition.
/// ```
/// # use obsidian_export::{Context, Exporter, MarkdownEvents, PostprocessorResult};
/// # use pulldown_cmark::{CowStr, Event};
/// # use std::path::PathBuf;
/// # use tempfile::TempDir;
/// #
/// /// This postprocessor replaces any instance of "foo" with "bar" in the note body.
/// fn foo_to_bar(context: &mut Context, events: &mut MarkdownEvents) -> PostprocessorResult {
///     for event in events.iter_mut() {
///         if let Event::Text(text) = event {
///             *event = Event::Text(CowStr::from(text.replace("foo", "bar")))
///         }
///     }
///     PostprocessorResult::Continue
/// }
///
/// # let tmp_dir = TempDir::new().expect("failed to make tempdir");
/// # let source = PathBuf::from("tests/testdata/input/postprocessors");
/// # let destination = tmp_dir.path().to_path_buf();
/// # let mut exporter = Exporter::new(source, destination);
/// exporter.add_postprocessor(&foo_to_bar);
/// # exporter.run().unwrap();
/// ```

pub type Postprocessor =
    dyn Fn(&mut Context, &mut MarkdownEvents) -> PostprocessorResult + Send + Sync;
type Result<T, E = ExportError> = std::result::Result<T, E>;

const PERCENTENCODE_CHARS: &AsciiSet = &CONTROLS.add(b' ').add(b'(').add(b')').add(b'%').add(b'?');
const NOTE_RECURSION_LIMIT: usize = 10;

#[non_exhaustive]
#[derive(Debug, Snafu)]
/// ExportError represents all errors which may be returned when using this crate.
pub enum ExportError {
    #[snafu(display("failed to read from '{}'", path.display()))]
    /// This occurs when a read IO operation fails.
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to write to '{}'", path.display()))]
    /// This occurs when a write IO operation fails.
    WriteError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Encountered an error while trying to walk '{}'", path.display()))]
    /// This occurs when an error is encountered while trying to walk a directory.
    WalkDirError {
        path: PathBuf,
        source: ignore::Error,
    },

    #[snafu(display("No such file or directory: {}", path.display()))]
    /// This occurs when an operation is requested on a file or directory which does not exist.
    PathDoesNotExist { path: PathBuf },

    #[snafu(display("Invalid character encoding encountered"))]
    /// This error may occur when invalid UTF8 is encountered.
    ///
    /// Currently, operations which assume UTF8 perform lossy encoding however.
    CharacterEncodingError { source: str::Utf8Error },

    #[snafu(display("Recursion limit exceeded"))]
    /// This error occurs when embedded notes are too deeply nested or cause an infinite loop.
    ///
    /// When this happens, `file_tree` contains a list of all the files which were processed
    /// leading up to this error.
    RecursionLimitExceeded { file_tree: Vec<PathBuf> },

    #[snafu(display("Failed to export '{}'", path.display()))]
    /// This occurs when a file fails to export successfully.
    FileExportError {
        path: PathBuf,
        #[snafu(source(from(ExportError, Box::new)))]
        source: Box<ExportError>,
    },

    #[snafu(display("Failed to decode YAML frontmatter in '{}'", path.display()))]
    FrontMatterDecodeError {
        path: PathBuf,
        #[snafu(source(from(serde_yaml::Error, Box::new)))]
        source: Box<serde_yaml::Error>,
    },

    #[snafu(display("Failed to encode YAML frontmatter for '{}'", path.display()))]
    FrontMatterEncodeError {
        path: PathBuf,
        #[snafu(source(from(serde_yaml::Error, Box::new)))]
        source: Box<serde_yaml::Error>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Emitted by [Postprocessor]s to signal the next action to take.
pub enum PostprocessorResult {
    /// Continue with the next post-processor (if any).
    Continue,
    /// Use this note, but don't run any more post-processors after this one.
    StopHere,
    /// Skip this note (don't export it) and don't run any more post-processors.
    StopAndSkipNote,
}

#[derive(Clone)]
/// Exporter provides the main interface to this library.
///
/// Users are expected to create an Exporter using [`Exporter::new`], optionally followed by
/// customization using [`Exporter::frontmatter_strategy`] and [`Exporter::walk_options`].
///
/// After that, calling [`Exporter::run`] will start the export process.
pub struct Exporter<'a> {
    root: PathBuf,
    destination: PathBuf,
    start_at: PathBuf,
    frontmatter_strategy: FrontmatterStrategy,
    vault_contents: Option<Vec<PathBuf>>,
    walk_options: WalkOptions<'a>,
    process_embeds_recursively: bool,
    add_titles: bool,
    strip_comments: bool,
    postprocessors: Vec<&'a Postprocessor>,
    embed_postprocessors: Vec<&'a Postprocessor>,
}

impl<'a> fmt::Debug for Exporter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WalkOptions")
            .field("root", &self.root)
            .field("destination", &self.destination)
            .field("frontmatter_strategy", &self.frontmatter_strategy)
            .field("vault_contents", &self.vault_contents)
            .field("walk_options", &self.walk_options)
            .field(
                "process_embeds_recursively",
                &self.process_embeds_recursively,
            )
            .field("add_titles", &self.add_titles)
            .field("strip_comments", &self.strip_comments)
            .field(
                "postprocessors",
                &format!("<{} postprocessors active>", self.postprocessors.len()),
            )
            .field(
                "embed_postprocessors",
                &format!(
                    "<{} postprocessors active>",
                    self.embed_postprocessors.len()
                ),
            )
            .finish()
    }
}

impl<'a> Exporter<'a> {
    /// Create a new exporter which reads notes from `root` and exports these to
    /// `destination`.
    pub fn new(root: PathBuf, destination: PathBuf) -> Exporter<'a> {
        Exporter {
            start_at: root.clone(),
            root,
            destination,
            frontmatter_strategy: FrontmatterStrategy::Auto,
            walk_options: WalkOptions::default(),
            process_embeds_recursively: true,
            add_titles: false,
            strip_comments: false,
            vault_contents: None,
            postprocessors: vec![],
            embed_postprocessors: vec![],
        }
    }

    /// Set a custom starting point for the export.
    ///
    /// Normally all notes under `root` (except for notes excluded by ignore rules) will be exported.
    /// When `start_at` is set, only notes under this path will be exported to the target destination.
    pub fn start_at(&mut self, start_at: PathBuf) -> &mut Exporter<'a> {
        self.start_at = start_at;
        self
    }

    /// Set the [`WalkOptions`] to be used for this exporter.
    pub fn walk_options(&mut self, options: WalkOptions<'a>) -> &mut Exporter<'a> {
        self.walk_options = options;
        self
    }

    /// Set the [`FrontmatterStrategy`] to be used for this exporter.
    pub fn frontmatter_strategy(&mut self, strategy: FrontmatterStrategy) -> &mut Exporter<'a> {
        self.frontmatter_strategy = strategy;
        self
    }

    /// Set the behavior when recursive embeds are encountered.
    ///
    /// When `recursive` is true (the default), emdeds are always processed recursively. This may
    /// lead to infinite recursion when note A embeds B, but B also embeds A.
    /// (When this happens, [ExportError::RecursionLimitExceeded] will be returned by [Exporter::run]).
    ///
    /// When `recursive` is false, if a note is encountered for a second time while processing the
    /// original note, instead of embedding it again a link to the note is inserted instead.
    pub fn process_embeds_recursively(&mut self, recursive: bool) -> &mut Exporter<'a> {
        self.process_embeds_recursively = recursive;
        self
    }

    /// Enable or disable addition of title headings to the top of each parsed note
    ///
    /// When this option is enabled, each parsed note will be considered to start with a heading:
    ///
    ///   # Title-of-note
    ///
    /// even when no such line is present in the source file. The title is inferred based on the
    /// filename of the note.
    ///
    /// This option makes heavily nested note embeds make more sense in the exported document,
    /// since it shows which note the embedded content comes from. It loosely matches the behaviour
    /// of the mainline Obsidian UI when viewing notes in preview mode.
    pub fn add_titles(&mut self, add_titles: bool) -> &mut Exporter<'a> {
        self.add_titles = add_titles;
        self
    }

    /// Strip comment lines from output.
    ///
    /// A comment line is one which begins with the characters "%%".
    pub fn strip_comments(&mut self, strip_comments: bool) -> &mut Exporter<'a> {
        self.strip_comments = strip_comments;
        self
    }

    /// Append a function to the chain of [postprocessors][Postprocessor] to run on exported Obsidian Markdown notes.
    pub fn add_postprocessor(&mut self, processor: &'a Postprocessor) -> &mut Exporter<'a> {
        self.postprocessors.push(processor);
        self
    }

    /// Append a function to the chain of [postprocessors][Postprocessor] for embeds.
    pub fn add_embed_postprocessor(&mut self, processor: &'a Postprocessor) -> &mut Exporter<'a> {
        self.embed_postprocessors.push(processor);
        self
    }

    /// Export notes using the settings configured on this exporter.
    pub fn run(&mut self) -> Result<()> {
        if !self.root.exists() {
            return Err(ExportError::PathDoesNotExist {
                path: self.root.clone(),
            });
        }

        self.vault_contents = Some(vault_contents(
            self.root.as_path(),
            self.walk_options.clone(),
        )?);

        // When a single file is specified, just need to export that specific file instead of
        // iterating over all discovered files. This also allows us to accept destination as either
        // a file or a directory name.
        if self.root.is_file() || self.start_at.is_file() {
            let source_filename = self
                .start_at
                .file_name()
                .expect("File without a filename? How is that possible?")
                .to_string_lossy();

            let destination = match self.destination.is_dir() {
                true => self.destination.join(String::from(source_filename)),
                false => {
                    let parent = self.destination.parent().unwrap_or(&self.destination);
                    // Avoid recursively creating self.destination through the call to
                    // export_note when the parent directory doesn't exist.
                    if !parent.exists() {
                        return Err(ExportError::PathDoesNotExist {
                            path: parent.to_path_buf(),
                        });
                    }
                    self.destination.clone()
                }
            };
            return self.export_note(&self.start_at, &destination);
        }

        if !self.destination.exists() {
            return Err(ExportError::PathDoesNotExist {
                path: self.destination.clone(),
            });
        }
        self.vault_contents
            .as_ref()
            .unwrap()
            .clone()
            .into_par_iter()
            .filter(|file| file.starts_with(&self.start_at))
            .try_for_each(|file| {
                let relative_path = file
                    .strip_prefix(&self.start_at.clone())
                    .expect("file should always be nested under root")
                    .to_path_buf();
                let destination = &self.destination.join(&relative_path);
                self.export_note(&file, destination)
            })?;
        Ok(())
    }

    fn export_note(&self, src: &Path, dest: &Path) -> Result<()> {
        match is_markdown_file(src) {
            true => self.parse_and_export_obsidian_note(src, dest),
            false => copy_file(src, dest),
        }
        .context(FileExportError { path: src })
    }

    fn parse_and_export_obsidian_note(&self, src: &Path, dest: &Path) -> Result<()> {
        let mut context = Context::new(src.to_path_buf(), dest.to_path_buf());

        let (frontmatter, mut markdown_events) = self.parse_obsidian_note(src, &context)?;
        context.frontmatter = frontmatter;
        for func in &self.postprocessors {
            match func(&mut context, &mut markdown_events) {
                PostprocessorResult::StopHere => break,
                PostprocessorResult::StopAndSkipNote => return Ok(()),
                PostprocessorResult::Continue => (),
            }
        }

        let dest = context.destination;
        let mut outfile = create_file(&dest)?;
        let write_frontmatter = match self.frontmatter_strategy {
            FrontmatterStrategy::Always => true,
            FrontmatterStrategy::Never => false,
            FrontmatterStrategy::Auto => !context.frontmatter.is_empty(),
        };
        if write_frontmatter {
            let mut frontmatter_str = frontmatter_to_str(context.frontmatter)
                .context(FrontMatterEncodeError { path: src })?;
            frontmatter_str.push('\n');
            outfile
                .write_all(frontmatter_str.as_bytes())
                .context(WriteError { path: &dest })?;
        }
        outfile
            .write_all(render_mdevents_to_mdtext(markdown_events).as_bytes())
            .context(WriteError { path: &dest })?;
        Ok(())
    }

    fn parse_obsidian_note<'b>(
        &self,
        path: &Path,
        context: &Context,
    ) -> Result<(Frontmatter, MarkdownEvents<'b>)> {
        if context.note_depth() > NOTE_RECURSION_LIMIT {
            return Err(ExportError::RecursionLimitExceeded {
                file_tree: context.file_tree(),
            });
        }
        let content = fs::read_to_string(&path).context(ReadError { path })?;
        let (frontmatter, content) =
            matter::matter(&content).unwrap_or(("".to_string(), content.to_string()));
        let frontmatter =
            frontmatter_from_str(&frontmatter).context(FrontMatterDecodeError { path })?;

        let mut parser_options = Options::empty();
        parser_options.insert(Options::ENABLE_TABLES);
        parser_options.insert(Options::ENABLE_FOOTNOTES);
        parser_options.insert(Options::ENABLE_STRIKETHROUGH);
        parser_options.insert(Options::ENABLE_TASKLISTS);

        let mut ref_parser = RefParser::new();
        let mut events = vec![];
        // Most of the time, a reference triggers 5 events: [ or ![, [, <text>, ], ]
        let mut buffer = Vec::with_capacity(5);

        if self.add_titles {
            // Ensure that each (possibly embedded) note starts with a reasonable top-level heading
            let note_name = infer_note_title_from_path(path);
            let h1_tag = Tag::Heading(HeadingLevel::H1, None, vec![]);

            events.push(Event::Start(h1_tag.clone()));
            events.push(Event::Text(note_name));
            events.push(Event::End(h1_tag.clone()));
        }

        for event in Parser::new_ext(&content, parser_options) {
            if ref_parser.state == RefParserState::Resetting {
                events.append(&mut buffer);
                buffer.clear();
                ref_parser.reset();
            }
            buffer.push(event.clone());
            match ref_parser.state {
                RefParserState::NoState => {
                    match event {
                        Event::Text(CowStr::Borrowed("![")) => {
                            ref_parser.ref_type = Some(RefType::Embed);
                            ref_parser.transition(RefParserState::ExpectSecondOpenBracket);
                        }
                        Event::Text(CowStr::Borrowed("[")) => {
                            ref_parser.ref_type = Some(RefType::Link);
                            ref_parser.transition(RefParserState::ExpectSecondOpenBracket);
                        }
                        Event::Text(ref s) => {
                            // TODO: This only handles %% at beginning of line, and doesn't handle
                            // other formatting embedded in comments
                            if self.strip_comments && !s.starts_with("%%") {
                                events.push(event);
                            }
                            buffer.clear();
                        }
                        _ => {
                            events.push(event);
                            buffer.clear();
                        },
                    };
                }
                RefParserState::ExpectSecondOpenBracket => match event {
                    Event::Text(CowStr::Borrowed("[")) => {
                        ref_parser.transition(RefParserState::ExpectRefText);
                    }
                    _ => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                },
                RefParserState::ExpectRefText => match event {
                    Event::Text(CowStr::Borrowed("]")) => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                    Event::Text(text) => {
                        ref_parser.ref_text.push_str(&text);
                        ref_parser.transition(RefParserState::ExpectRefTextOrCloseBracket);
                    }
                    _ => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                },
                RefParserState::ExpectRefTextOrCloseBracket => match event {
                    Event::Text(CowStr::Borrowed("]")) => {
                        ref_parser.transition(RefParserState::ExpectFinalCloseBracket);
                    }
                    Event::Text(text) => {
                        ref_parser.ref_text.push_str(&text);
                    }
                    _ => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                },
                RefParserState::ExpectFinalCloseBracket => match event {
                    Event::Text(CowStr::Borrowed("]")) => match ref_parser.ref_type {
                        Some(RefType::Link) => {
                            let mut elements = self.make_link_to_file(
                                ObsidianNoteReference::from_str(
                                    ref_parser.ref_text.clone().as_ref()
                                ),
                                context,
                            );
                            events.append(&mut elements);
                            buffer.clear();
                            ref_parser.transition(RefParserState::Resetting);
                        }
                        Some(RefType::Embed) => {
                            let mut elements = self.embed_file(
                                ref_parser.ref_text.clone().as_ref(),
                                context
                            )?;
                            events.append(&mut elements);
                            buffer.clear();
                            ref_parser.transition(RefParserState::Resetting);
                        }
                        None => panic!("In state ExpectFinalCloseBracket but ref_type is None"),
                    },
                    _ => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                },
                RefParserState::Resetting => panic!("Reached Resetting state, but it should have been handled prior to this match block"),
            }
        }
        if !buffer.is_empty() {
            events.append(&mut buffer);
        }
        Ok((
            frontmatter,
            events.into_iter().map(event_to_owned).collect(),
        ))
    }

    // Generate markdown elements for a file that is embedded within another note.
    //
    // - If the file being embedded is a note, it's content is included at the point of embed.
    // - If the file is an image, an image tag is generated.
    // - For other types of file, a regular link is created instead.
    fn embed_file<'b>(
        &self,
        link_text: &'a str,
        context: &'a Context,
    ) -> Result<MarkdownEvents<'b>> {
        let note_ref = ObsidianNoteReference::from_str(link_text);

        let path = match note_ref.file {
            Some(file) => lookup_filename_in_vault(file, self.vault_contents.as_ref().unwrap()),

            // If we have None file it is either to a section or id within the same file and thus
            // the current embed logic will fail, recurssing until it reaches it's limit.
            // For now we just bail early.
            None => return Ok(self.make_link_to_file(note_ref, context)),
        };

        if path.is_none() {
            // TODO: Extract into configurable function.
            eprintln!(
                "Warning: Unable to find embedded note\n\tReference: '{}'\n\tSource: '{}'\n",
                note_ref
                    .file
                    .unwrap_or_else(|| context.current_file().to_str().unwrap()),
                context.current_file().display(),
            );
            return Ok(vec![]);
        }

        let path = path.unwrap();
        let mut child_context = Context::from_parent(context, path);
        let no_ext = OsString::new();

        if !self.process_embeds_recursively && context.file_tree().contains(path) {
            return Ok([
                vec![Event::Text(CowStr::Borrowed("→ "))],
                self.make_link_to_file(note_ref, &child_context),
            ]
            .concat());
        }

        let events = match path.extension().unwrap_or(&no_ext).to_str() {
            Some("md") => {
                let (frontmatter, mut events) = self.parse_obsidian_note(path, &child_context)?;
                child_context.frontmatter = frontmatter;
                if let Some(section) = note_ref.section {
                    events = reduce_to_section(events, section);
                }
                for func in &self.embed_postprocessors {
                    // Postprocessors running on embeds shouldn't be able to change frontmatter (or
                    // any other metadata), so we give them a clone of the context.
                    match func(&mut child_context, &mut events) {
                        PostprocessorResult::StopHere => break,
                        PostprocessorResult::StopAndSkipNote => {
                            events = vec![];
                        }
                        PostprocessorResult::Continue => (),
                    }
                }
                events
            }
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("svg") => {
                self.make_link_to_file(note_ref, &child_context)
                    .into_iter()
                    .map(|event| match event {
                        // make_link_to_file returns a link to a file. With this we turn the link
                        // into an image reference instead. Slightly hacky, but avoids needing
                        // to keep another utility function around for this, or introducing an
                        // extra parameter on make_link_to_file.
                        Event::Start(Tag::Link(linktype, cowstr1, cowstr2)) => {
                            Event::Start(Tag::Image(
                                linktype,
                                CowStr::from(cowstr1.into_string()),
                                CowStr::from(cowstr2.into_string()),
                            ))
                        }
                        Event::End(Tag::Link(linktype, cowstr1, cowstr2)) => {
                            Event::End(Tag::Image(
                                linktype,
                                CowStr::from(cowstr1.into_string()),
                                CowStr::from(cowstr2.into_string()),
                            ))
                        }
                        _ => event,
                    })
                    .collect()
            }
            _ => self.make_link_to_file(note_ref, &child_context),
        };
        Ok(events)
    }

    fn make_link_to_file<'b, 'c>(
        &self,
        reference: ObsidianNoteReference<'b>,
        context: &Context,
    ) -> MarkdownEvents<'c> {
        let target_file = reference
            .file
            .map(|file| lookup_filename_in_vault(file, self.vault_contents.as_ref().unwrap()))
            .unwrap_or_else(|| Some(context.current_file()));

        if target_file.is_none() {
            // TODO: Extract into configurable function.
            eprintln!(
                "Warning: Unable to find referenced note\n\tReference: '{}'\n\tSource: '{}'\n",
                reference
                    .file
                    .unwrap_or_else(|| context.current_file().to_str().unwrap()),
                context.current_file().display(),
            );
            let dummy_link_tag = pulldown_cmark::Tag::Link(
                pulldown_cmark::LinkType::Inline,
                CowStr::from(""),
                CowStr::from(""),
            );
            return vec![
                Event::Start(dummy_link_tag.clone()),
                Event::Text(CowStr::from(reference.display())),
                Event::End(dummy_link_tag.clone()),
            ];
        }
        let target_file = target_file.unwrap();
        // We use root_file() rather than current_file() here to make sure links are always
        // relative to the outer-most note, which is the note which this content is inserted into
        // in case of embedded notes.
        let rel_link = diff_paths(
            target_file,
            &context
                .root_file()
                .parent()
                .expect("obsidian content files should always have a parent"),
        )
        .expect("should be able to build relative path when target file is found in vault");

        let rel_link = rel_link.to_string_lossy();
        let mut link = utf8_percent_encode(&rel_link, PERCENTENCODE_CHARS).to_string();

        if let Some(section) = reference.section {
            link.push('#');
            link.push_str(&slugify(section));
        }

        let link_tag = pulldown_cmark::Tag::Link(
            pulldown_cmark::LinkType::Inline,
            CowStr::from(link),
            CowStr::from(""),
        );

        vec![
            Event::Start(link_tag.clone()),
            Event::Text(CowStr::from(reference.display())),
            Event::End(link_tag.clone()),
        ]
    }
}

fn lookup_filename_in_vault<'a>(
    filename: &str,
    vault_contents: &'a [PathBuf],
) -> Option<&'a PathBuf> {
    // Markdown files don't have their .md extension added by Obsidian, but other files (images,
    // PDFs, etc) do so we match on both possibilities.
    //
    // References can also refer to notes in a different case (to lowercase text in a
    // sentence even if the note is capitalized for example) so we also try a case-insensitive
    // lookup.
    vault_contents.iter().find(|path| {
        let path_lowered = PathBuf::from(path.to_string_lossy().to_lowercase());
        path.ends_with(&filename)
            || path_lowered.ends_with(&filename.to_lowercase())
            || path.ends_with(format!("{}.md", &filename))
            || path_lowered.ends_with(format!("{}.md", &filename.to_lowercase()))
    })
}

fn infer_note_title_from_path(path: &Path) -> CowStr {
    const PLACEHOLDER_TITLE: &str = "invalid-note-title";

    match path.file_stem() {
        None => CowStr::from(PLACEHOLDER_TITLE),
        Some(s) => CowStr::from(s.to_string_lossy().into_owned()),
    }
}

fn render_mdevents_to_mdtext(markdown: MarkdownEvents) -> String {
    let mut buffer = String::new();
    cmark_with_options(
        markdown.iter(),
        &mut buffer,
        None,
        pulldown_cmark_to_cmark::Options::default(),
    )
    .expect("formatting to string not expected to fail");
    buffer.push('\n');
    buffer
}

fn create_file(dest: &Path) -> Result<File> {
    let file = File::create(&dest)
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                let parent = dest.parent().expect("file should have a parent directory");
                if let Err(err) = std::fs::create_dir_all(&parent) {
                    return Err(err);
                }
            }
            File::create(&dest)
        })
        .context(WriteError { path: dest })?;
    Ok(file)
}

fn copy_file(src: &Path, dest: &Path) -> Result<()> {
    std::fs::copy(&src, &dest)
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                let parent = dest.parent().expect("file should have a parent directory");
                if let Err(err) = std::fs::create_dir_all(&parent) {
                    return Err(err);
                }
            }
            std::fs::copy(&src, &dest)
        })
        .context(WriteError { path: dest })?;
    Ok(())
}

fn is_markdown_file(file: &Path) -> bool {
    let no_ext = OsString::new();
    let ext = file.extension().unwrap_or(&no_ext).to_string_lossy();
    ext == "md"
}

/// Reduce a given `MarkdownEvents` to just those elements which are children of the given section
/// (heading name).
fn reduce_to_section<'a, 'b>(events: MarkdownEvents<'a>, section: &'b str) -> MarkdownEvents<'a> {
    let mut filtered_events = Vec::with_capacity(events.len());
    let mut target_section_encountered = false;
    let mut currently_in_target_section = false;
    let mut section_level = HeadingLevel::H1;
    let mut last_level = HeadingLevel::H1;
    let mut last_tag_was_heading = false;

    for event in events.into_iter() {
        filtered_events.push(event.clone());
        match event {
            // FIXME: This should propagate fragment_identifier and classes.
            Event::Start(Tag::Heading(level, _fragment_identifier, _classes)) => {
                last_tag_was_heading = true;
                last_level = level;
                if currently_in_target_section && level <= section_level {
                    currently_in_target_section = false;
                    filtered_events.pop();
                }
            }
            Event::Text(cowstr) => {
                if !last_tag_was_heading {
                    last_tag_was_heading = false;
                    continue;
                }
                last_tag_was_heading = false;

                if cowstr.to_string().to_lowercase() == section.to_lowercase() {
                    target_section_encountered = true;
                    currently_in_target_section = true;
                    section_level = last_level;

                    let current_event = filtered_events.pop().unwrap();
                    let heading_start_event = filtered_events.pop().unwrap();
                    filtered_events.clear();
                    filtered_events.push(heading_start_event);
                    filtered_events.push(current_event);
                }
            }
            _ => {}
        }
        if target_section_encountered && !currently_in_target_section {
            return filtered_events;
        }
    }
    filtered_events
}

fn event_to_owned<'a>(event: Event) -> Event<'a> {
    match event {
        Event::Start(tag) => Event::Start(tag_to_owned(tag)),
        Event::End(tag) => Event::End(tag_to_owned(tag)),
        Event::Text(cowstr) => Event::Text(CowStr::from(cowstr.into_string())),
        Event::Code(cowstr) => Event::Code(CowStr::from(cowstr.into_string())),
        Event::Html(cowstr) => Event::Html(CowStr::from(cowstr.into_string())),
        Event::FootnoteReference(cowstr) => {
            Event::FootnoteReference(CowStr::from(cowstr.into_string()))
        }
        Event::SoftBreak => Event::SoftBreak,
        Event::HardBreak => Event::HardBreak,
        Event::Rule => Event::Rule,
        Event::TaskListMarker(checked) => Event::TaskListMarker(checked),
    }
}

fn tag_to_owned<'a>(tag: Tag) -> Tag<'a> {
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading(level, _fragment_identifier, _classes) => {
            // FIXME: This should propagate fragment_identifier and classes.
            Tag::Heading(level, None, Vec::new())
        }
        Tag::BlockQuote => Tag::BlockQuote,
        Tag::CodeBlock(codeblock_kind) => Tag::CodeBlock(codeblock_kind_to_owned(codeblock_kind)),
        Tag::List(optional) => Tag::List(optional),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(cowstr) => {
            Tag::FootnoteDefinition(CowStr::from(cowstr.into_string()))
        }
        Tag::Table(alignment_vector) => Tag::Table(alignment_vector),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link(linktype, cowstr1, cowstr2) => Tag::Link(
            linktype,
            CowStr::from(cowstr1.into_string()),
            CowStr::from(cowstr2.into_string()),
        ),
        Tag::Image(linktype, cowstr1, cowstr2) => Tag::Image(
            linktype,
            CowStr::from(cowstr1.into_string()),
            CowStr::from(cowstr2.into_string()),
        ),
    }
}

fn codeblock_kind_to_owned<'a>(codeblock_kind: CodeBlockKind) -> CodeBlockKind<'a> {
    match codeblock_kind {
        CodeBlockKind::Indented => CodeBlockKind::Indented,
        CodeBlockKind::Fenced(cowstr) => CodeBlockKind::Fenced(CowStr::from(cowstr.into_string())),
    }
}
