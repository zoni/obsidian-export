#[macro_use]
extern crate lazy_static;

mod walker;

pub use walker::{vault_contents, WalkOptions};

use pathdiff::diff_paths;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark_with_options;
use rayon::prelude::*;
use regex::Regex;
use snafu::{ResultExt, Snafu};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::str;

type Result<T, E = ExportError> = std::result::Result<T, E>;
type MarkdownTree<'a> = Vec<Event<'a>>;

lazy_static! {
    static ref OBSIDIAN_NOTE_LINK_RE: Regex =
        Regex::new(r"^(?P<file>[^#|]+)(#(?P<block>.+?))??(\|(?P<label>.+?))??$").unwrap();
}
const PERCENTENCODE_CHARS: &AsciiSet = &CONTROLS.add(b' ').add(b'(').add(b')').add(b'%');
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
}

#[derive(Debug, Clone, Copy)]
/// FrontmatterStrategy determines how frontmatter is handled in Markdown files.
pub enum FrontmatterStrategy {
    /// Copy frontmatter when a note has frontmatter defined.
    Auto,
    /// Always add frontmatter header, including empty frontmatter when none was originally
    /// specified.
    Always,
    /// Never add any frontmatter to notes.
    Never,
}

#[derive(Debug, Clone)]
/// Exporter provides the main interface to this library.
///
/// Users are expected to create an Exporter using [`Exporter::new`], optionally followed by
/// customization using [`Exporter::frontmatter_strategy`] and [`Exporter::walk_options`].
///
/// After that, calling [`Exporter::run`] will start the export process.
pub struct Exporter<'a> {
    root: PathBuf,
    destination: PathBuf,
    frontmatter_strategy: FrontmatterStrategy,
    vault_contents: Option<Vec<PathBuf>>,
    walk_options: WalkOptions<'a>,
}

#[derive(Debug, Clone)]
/// Context holds parser metadata for the file/note currently being parsed.
struct Context {
    file_tree: Vec<PathBuf>,
    frontmatter_strategy: FrontmatterStrategy,
}

impl Context {
    /// Create a new `Context`
    fn new(file: PathBuf) -> Context {
        Context {
            file_tree: vec![file.clone()],
            frontmatter_strategy: FrontmatterStrategy::Auto,
        }
    }

    /// Create a new `Context` which inherits from a parent Context.
    fn from_parent(context: &Context, child: &PathBuf) -> Context {
        let mut context = context.clone();
        context.file_tree.push(child.to_path_buf());
        context
    }

    /// Associate a new `FrontmatterStrategy` with this context.
    fn set_frontmatter_strategy(&mut self, strategy: FrontmatterStrategy) -> &mut Context {
        self.frontmatter_strategy = strategy;
        self
    }

    /// Return the path of the file currently being parsed.
    fn current_file(&self) -> &PathBuf {
        self.file_tree
            .last()
            .expect("Context not initialized properly, file_tree is empty")
    }

    /// Return the path of the root file.
    ///
    /// Typically this will yield the same element as `current_file`, but when a note is embedded
    /// within another note, this will return the outer-most note.
    fn root_file(&self) -> &PathBuf {
        self.file_tree
            .first()
            .expect("Context not initialized properly, file_tree is empty")
    }

    /// Return the note depth (nesting level) for this context.
    fn note_depth(&self) -> usize {
        self.file_tree.len()
    }

    /// Return the list of files associated with this context.
    ///
    /// The first element corresponds to the root file, the final element corresponds to the file
    /// which is currently being processed (see also `current_file`).
    fn file_tree(&self) -> Vec<PathBuf> {
        self.file_tree.clone()
    }
}

impl<'a> Exporter<'a> {
    /// Create a new exporter which reads notes from `source` and exports these to
    /// `destination`.
    pub fn new(source: PathBuf, destination: PathBuf) -> Exporter<'a> {
        Exporter {
            root: source,
            destination,
            frontmatter_strategy: FrontmatterStrategy::Auto,
            walk_options: WalkOptions::default(),
            vault_contents: None,
        }
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

    /// Export notes using the settings configured on this exporter.
    pub fn run(&mut self) -> Result<()> {
        if !self.root.exists() {
            return Err(ExportError::PathDoesNotExist {
                path: self.root.clone(),
            });
        }

        // When a single file is specified, we can short-circuit contruction of walk and associated
        // directory traversal. This also allows us to accept destination as either a file or a
        // directory name.
        if self.root.is_file() {
            self.vault_contents = Some(vec![self.root.clone()]);
            let source_filename = self
                .root
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
            return Ok(self.export_note(&self.root, &destination)?);
        }

        if !self.destination.exists() {
            return Err(ExportError::PathDoesNotExist {
                path: self.destination.clone(),
            });
        }

        self.vault_contents = Some(vault_contents(
            self.root.as_path(),
            self.walk_options.clone(),
        )?);
        self.vault_contents
            .as_ref()
            .unwrap()
            .clone()
            .into_par_iter()
            .try_for_each(|file| {
                let relative_path = file
                    .strip_prefix(&self.root.clone())
                    .expect("file should always be nested under root")
                    .to_path_buf();
                let destination = &self.destination.join(&relative_path);
                self.export_note(&file, destination)
            })?;
        Ok(())
    }

    fn export_note(&self, src: &Path, dest: &Path) -> Result<()> {
        match is_markdown_file(src) {
            true => self.parse_and_export_obsidian_note(src, dest, self.frontmatter_strategy),
            false => copy_file(src, dest),
        }
        .context(FileExportError { path: src })
    }

    fn parse_and_export_obsidian_note(
        &self,
        src: &Path,
        dest: &Path,
        frontmatter_strategy: FrontmatterStrategy,
    ) -> Result<()> {
        let content = fs::read_to_string(&src).context(ReadError { path: src })?;

        let (mut frontmatter, _content) =
            matter::matter(&content).unwrap_or(("".to_string(), content.to_string()));
        frontmatter = frontmatter.trim().to_string();
        //let mut outfile = create_file(&dest).context(FileIOError { filename: dest })?;
        let mut outfile = create_file(&dest)?;

        let write_frontmatter = match frontmatter_strategy {
            FrontmatterStrategy::Always => true,
            FrontmatterStrategy::Never => false,
            FrontmatterStrategy::Auto => frontmatter != "",
        };
        if write_frontmatter {
            if frontmatter != "" && !frontmatter.ends_with('\n') {
                frontmatter.push('\n');
            }
            outfile
                .write_all(format!("---\n{}---\n\n", frontmatter).as_bytes())
                .context(WriteError { path: &dest })?;
        }

        let mut context = Context::new(src.to_path_buf());
        context.set_frontmatter_strategy(frontmatter_strategy);
        let markdown_tree = self.parse_obsidian_note(&src, &context)?;
        outfile
            .write_all(render_mdtree_to_mdtext(markdown_tree).as_bytes())
            .context(WriteError { path: &dest })?;
        Ok(())
    }

    fn parse_obsidian_note<'b>(&self, path: &Path, context: &Context) -> Result<MarkdownTree<'b>> {
        if context.note_depth() > NOTE_RECURSION_LIMIT {
            return Err(ExportError::RecursionLimitExceeded {
                file_tree: context.file_tree(),
            });
        }
        let content = fs::read_to_string(&path).context(ReadError { path })?;
        let (_frontmatter, content) =
            matter::matter(&content).unwrap_or(("".to_string(), content.to_string()));

        let mut parser_options = Options::empty();
        parser_options.insert(Options::ENABLE_TABLES);
        parser_options.insert(Options::ENABLE_FOOTNOTES);
        parser_options.insert(Options::ENABLE_STRIKETHROUGH);
        parser_options.insert(Options::ENABLE_TASKLISTS);

        // Use of ENABLE_SMART_PUNCTUATION causes character replacements which breaks up the single
        // Event::Text element that is emitted between `[[` and `]]` into an unpredictable number of
        // additional elements. This completely throws off the current parsing strategy and is
        // unsupported. If a user were to want this, a strategy would be to do a second-stage pass over
        // the rewritten markdown just before feeding to pulldown_cmark_to_cmark.
        //parser_options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let mut parser = Parser::new_ext(&content, parser_options);
        let mut tree = vec![];
        let mut buffer = Vec::with_capacity(5);

        while let Some(event) = parser.next() {
            match event {
                Event::Text(CowStr::Borrowed("[")) | Event::Text(CowStr::Borrowed("![")) => {
                    buffer.push(event);
                    // A lone '[' or a '![' Text event signifies the possible start of a linked or
                    // embedded note. However, we cannot be sure unless it's also followed by another
                    // '[', the note reference itself and closed by a double ']'. To determine whether
                    // that's the case, we need to read ahead so we can backtrack later if needed.
                    for _ in 1..5 {
                        if let Some(event) = parser.next() {
                            buffer.push(event);
                        }
                    }
                    if buffer.len() != 5
                    // buffer[0] has '[' or '![', but we already know this
                    || buffer[1] != Event::Text(CowStr::Borrowed("["))
                    || buffer[3] != Event::Text(CowStr::Borrowed("]"))
                    || buffer[4] != Event::Text(CowStr::Borrowed("]"))
                    {
                        tree.append(&mut buffer);
                        buffer.clear();
                        continue;
                    }

                    if let Event::Text(CowStr::Borrowed(text)) = buffer[2] {
                        match buffer[0] {
                            Event::Text(CowStr::Borrowed("[")) => {
                                let mut link_events =
                                    self.obsidian_note_link_to_markdown(&text, context);
                                tree.append(&mut link_events);
                                buffer.clear();
                                continue;
                            }
                            Event::Text(CowStr::Borrowed("![")) => {
                                let mut elements = self.embed_file(&text, &context)?;
                                tree.append(&mut elements);
                                buffer.clear();
                                continue;
                            }
                            // This arm can never be reached due to the outer match against event, but
                            // the compiler (currently) cannot infer this.
                            _ => {}
                        }
                    }
                }
                _ => tree.push(event),
            }
            if !buffer.is_empty() {
                tree.append(&mut buffer);
                buffer.clear();
            }
        }
        tree.append(&mut buffer);
        Ok(tree.into_iter().map(event_to_owned).collect())
    }

    // Generate markdown elements for a file that is embedded within another note.
    //
    // - If the file being embedded is a note, it's content is included at the point of embed.
    // - If the file is an image, an image tag is generated.
    // - For other types of file, a regular link is created instead.
    fn embed_file<'b>(&self, note_name: &'a str, context: &'a Context) -> Result<MarkdownTree<'a>> {
        // TODO: If a #section is specified, reduce returned MarkdownTree to just
        // that section.
        let note_name = note_name.split('#').collect::<Vec<&str>>()[0];

        let tree = match lookup_filename_in_vault(note_name, &self.vault_contents.as_ref().unwrap())
        {
            Some(path) => {
                let context = Context::from_parent(context, path);
                let no_ext = OsString::new();
                match path.extension().unwrap_or(&no_ext).to_str() {
                    Some("md") => self.parse_obsidian_note(&path, &context)?,
                    Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") => {
                        self.make_link_to_file(&note_name, &note_name, &context)
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
                    _ => self.make_link_to_file(&note_name, &note_name, &context),
                }
            }
            None => {
                // TODO: Extract into configurable function.
                println!(
                    "Warning: Unable to find embedded note\n\tReference: '{}'\n\tSource: '{}'\n",
                    note_name,
                    context.current_file().display(),
                );
                vec![]
            }
        };
        Ok(tree)
    }

    fn obsidian_note_link_to_markdown(&self, content: &'a str, context: &Context) -> MarkdownTree {
        let captures = OBSIDIAN_NOTE_LINK_RE
            .captures(&content)
            .expect("note link regex didn't match - bad input?");
        let notename = captures
            .name("file")
            .expect("Obsidian links should always reference a file");
        let label = captures.name("label").unwrap_or(notename);

        self.make_link_to_file(notename.as_str(), label.as_str(), context)
    }

    fn make_link_to_file<'b>(
        &self,
        file: &'b str,
        label: &'b str,
        context: &Context,
    ) -> MarkdownTree<'b> {
        let target_file = lookup_filename_in_vault(file, &self.vault_contents.as_ref().unwrap());
        if target_file.is_none() {
            // TODO: Extract into configurable function.
            println!(
                "Warning: Unable to find referenced note\n\tReference: '{}'\n\tSource: '{}'\n",
                file,
                context.current_file().display(),
            );
            return vec![
                Event::Start(Tag::Emphasis),
                Event::Text(CowStr::from(String::from(label))),
                Event::End(Tag::Emphasis),
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
        let encoded_link = utf8_percent_encode(&rel_link, PERCENTENCODE_CHARS);

        let link = pulldown_cmark::Tag::Link(
            pulldown_cmark::LinkType::Inline,
            CowStr::from(encoded_link.to_string()),
            CowStr::from(""),
        );

        vec![
            Event::Start(link.clone()),
            Event::Text(CowStr::from(label)),
            Event::End(link.clone()),
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
        path.ends_with(&filename)
            || path.ends_with(&filename.to_lowercase())
            || path.ends_with(format!("{}.md", &filename))
            || path.ends_with(format!("{}.md", &filename.to_lowercase()))
    })
}

fn render_mdtree_to_mdtext(markdown: MarkdownTree) -> String {
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
        Tag::Heading(level) => Tag::Heading(level),
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
