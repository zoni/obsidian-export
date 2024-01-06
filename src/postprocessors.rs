//! A collection of officially maintained [postprocessors][crate::Postprocessor].

use std::{
    collections::BTreeSet,
    fmt::DebugStruct,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use super::{Context, MarkdownEvents, PostprocessorResult, PERCENTENCODE_CHARS};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet};
use pulldown_cmark::{CowStr, Event, Tag};
use rayon::iter::{ParallelDrainRange, ParallelIterator};
use serde_yaml::Value;

/// This postprocessor converts all soft line breaks to hard line breaks. Enabling this mimics
/// Obsidian's _'Strict line breaks'_ setting.
pub fn softbreaks_to_hardbreaks(
    _context: &mut Context,
    events: &mut MarkdownEvents,
) -> PostprocessorResult {
    for event in events.iter_mut() {
        if event == &Event::SoftBreak {
            *event = Event::HardBreak;
        }
    }
    PostprocessorResult::Continue
}

pub fn filter_by_tags(
    skip_tags: Vec<String>,
    only_tags: Vec<String>,
) -> impl Fn(&mut Context, &mut MarkdownEvents) -> PostprocessorResult {
    move |context: &mut Context, _events: &mut MarkdownEvents| -> PostprocessorResult {
        match context.frontmatter.get("tags") {
            None => filter_by_tags_(&[], &skip_tags, &only_tags),
            Some(Value::Sequence(tags)) => filter_by_tags_(tags, &skip_tags, &only_tags),
            _ => PostprocessorResult::Continue,
        }
    }
}

fn filter_by_tags_(
    tags: &[Value],
    skip_tags: &[String],
    only_tags: &[String],
) -> PostprocessorResult {
    let skip = skip_tags
        .iter()
        .any(|tag| tags.contains(&Value::String(tag.to_string())));
    let include = only_tags.is_empty()
        || only_tags
            .iter()
            .any(|tag| tags.contains(&Value::String(tag.to_string())));

    if skip || !include {
        PostprocessorResult::StopAndSkipNote
    } else {
        PostprocessorResult::Continue
    }
}

pub struct SharedResolverState {
    depth: usize,
    current_depth: RwLock<usize>,
    files_to_parse: RwLock<BTreeSet<PathBuf>>,
    linked_files: Mutex<Vec<PathBuf>>,
}

impl SharedResolverState {
    pub fn new(depth: usize) -> Arc<SharedResolverState> {
        Arc::new(SharedResolverState {
            depth,
            current_depth: RwLock::new(0),
            files_to_parse: RwLock::new(BTreeSet::new()),
            linked_files: Mutex::new(Vec::new()),
        })
    }
    pub fn update_and_check_should_continue(&self) -> bool {
        let mut current_depth = self.current_depth.write().unwrap();

        if *current_depth < self.depth {
            *current_depth += 1;
            let mut files_to_parse = self.files_to_parse.write().unwrap();
            *files_to_parse = self
                .linked_files
                .lock()
                .unwrap()
                .par_drain(..)
                .collect::<BTreeSet<PathBuf>>();
            if !files_to_parse.is_empty() {
                return false;
            }
        }
        return true;
    }
}

pub struct RecursiveResolver {
    root: PathBuf,
    start_at: PathBuf,
    destination: PathBuf,
    shared_state: Arc<SharedResolverState>,
}

impl<'a: 'url, 'url> RecursiveResolver {
    pub fn new(
        root: PathBuf,
        start_at: PathBuf,
        destination: PathBuf,
        shared_state: Arc<SharedResolverState>,
    ) -> RecursiveResolver {
        RecursiveResolver {
            root,
            start_at,
            destination,

            shared_state: shared_state.clone(),
        }
    }

    pub fn start_at(&mut self, start_at: PathBuf) {
        self.start_at = start_at;
    }

    /// If this is the first iteration, links to files outside of start_at are changed so
    /// that they are to in the root of the destination
    pub fn postprocess(
        &self,
        context: &'a mut Context,
        events: &'url mut MarkdownEvents,
    ) -> PostprocessorResult {
        println!("postprocess: recursive_resolver");
        match *self.shared_state.current_depth.read().unwrap() == 0 {
            true => self.first_run(context, events),
            false => {
                if !self
                    .shared_state
                    .files_to_parse
                    .read()
                    .unwrap()
                    .contains(context.current_file())
                {
                    return PostprocessorResult::StopAndSkipNote;
                }
                self.other_runs(context, events)
            }
        }
    }

    fn first_run(
        &self,
        context: &'a mut Context,
        events: &'url mut MarkdownEvents,
    ) -> PostprocessorResult {
        //let path_changed = context.current_file() != &self.start_at;
        for event in events.iter_mut() {
            if let Event::Start(Tag::Link(_, url, _)) = event {
                println!("url: {}", url);
                if url.starts_with("https://") || url.starts_with("http://") {
                    continue;
                }

                let vault_path: PathBuf = get_vault_path(url, &self.start_at.as_path());
                println!("vault_path: {}", vault_path.to_string_lossy());
                // may still be within start_at
                if vault_path.starts_with(&self.start_at) {
                    continue;
                }

                if vault_path.exists() {
                    let vaultless_path = vault_path.strip_prefix(self.root.as_path()).unwrap();
                    set_url(url, self.destination.join(vaultless_path));
                    self.shared_state
                        .linked_files
                        .lock()
                        .unwrap()
                        .push(vault_path);
                }
            }
        }
        PostprocessorResult::Continue
    }

    fn other_runs(
        &self,
        context: &'a mut Context,
        events: &'url mut MarkdownEvents,
    ) -> PostprocessorResult {
        //let path_changed = context.current_file() != self.start_at;
        for event in events.iter_mut() {
            let relative_start = self.start_at.clone().strip_prefix(&self.root).unwrap();
            if let Event::Start(Tag::Link(_, url, _)) = event {
                if url.starts_with("https://") || url.starts_with("http://") {
                    continue;
                }
                let vault_path = get_vault_path(url, self.root.as_path());

                // if it's within start_at, we need to strip the difference between root and start_at

                //let vaultless_path = vault_path.strip_prefix(self.root.as_path()).unwrap();
                if vault_path.exists() {
                    if vault_path.starts_with(&self.start_at) {
                        let link_destination = self
                            .destination
                            .join(vault_path.strip_prefix(&self.start_at).unwrap());
                        set_url(url, link_destination);
                    }
                    if *self.shared_state.current_depth.read().unwrap() < self.shared_state.depth {
                        self.shared_state
                            .linked_files
                            .lock()
                            .unwrap()
                            .push(vault_path);
                    }
                }
            }
        }
        PostprocessorResult::Continue
    }
}
fn get_vault_path(url: &mut CowStr<'_>, root: &Path) -> PathBuf {
    let path_stub = PathBuf::from(
        percent_decode_str(url.as_ref())
            .decode_utf8()
            .unwrap()
            .as_ref(),
    );
    root.join(path_stub).canonicalize().unwrap()
}
fn set_url(url: &mut CowStr<'_>, link_destination: PathBuf) {
    *url = CowStr::from(
        utf8_percent_encode(
            &format!("{}", link_destination.to_string_lossy()),
            PERCENTENCODE_CHARS,
        )
        .to_string(),
    );
}

#[test]
fn test_filter_tags() {
    let tags = vec![
        Value::String("skip".to_string()),
        Value::String("publish".to_string()),
    ];
    let empty_tags = vec![];
    assert_eq!(
        filter_by_tags_(&empty_tags, &[], &[]),
        PostprocessorResult::Continue,
        "When no exclusion & inclusion are specified, files without tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &[]),
        PostprocessorResult::Continue,
        "When no exclusion & inclusion are specified, files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["exclude".to_string()], &[]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&empty_tags, &["exclude".to_string()], &[]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files without tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &["publish".to_string()]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&empty_tags, &[], &["include".to_string()]),
        PostprocessorResult::StopAndSkipNote,
        "When inclusion tags are specified files without tags are excluded"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &["include".to_string()]),
        PostprocessorResult::StopAndSkipNote,
        "When exclusion tags don't match files with tags are exluded"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["skip".to_string()], &["skip".to_string()]),
        PostprocessorResult::StopAndSkipNote,
        "When both inclusion and exclusion tags are the same exclusion wins"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["skip".to_string()], &["publish".to_string()]),
        PostprocessorResult::StopAndSkipNote,
        "When both inclusion and exclusion tags match exclusion wins"
    );
}
