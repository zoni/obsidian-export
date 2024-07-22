//! A collection of officially maintained [postprocessors][crate::Postprocessor].

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use super::{Context, MarkdownEvents, PostprocessorResult, PERCENTENCODE_CHARS};
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use pulldown_cmark::{CowStr, Event, Tag};
use rayon::iter::{ParallelDrainRange, ParallelExtend};
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
#[derive(Debug)]
pub struct SharedResolverState {
    depth: usize,
    current_depth: RwLock<usize>,
    files_to_parse: RwLock<BTreeSet<PathBuf>>,
    linked_files: Mutex<Vec<PathBuf>>,
    parsed_files: RwLock<BTreeSet<PathBuf>>,
}

impl SharedResolverState {
    pub fn new(depth: usize) -> Arc<SharedResolverState> {
        Arc::new(SharedResolverState {
            depth,
            current_depth: RwLock::new(0),
            files_to_parse: RwLock::new(BTreeSet::new()),
            linked_files: Mutex::new(Vec::new()),
            parsed_files: RwLock::new(BTreeSet::new()),
        })
    }
    pub fn update_and_check_should_continue(&self) -> bool {
        let mut current_depth = self.current_depth.write().unwrap();

        if *current_depth < self.depth {
            *current_depth += 1;

            let parsed_files = &mut *self.parsed_files.write().unwrap();

            let files_to_parse = &mut *self.files_to_parse.write().unwrap();
            parsed_files.append(files_to_parse);
            files_to_parse.par_extend(self.linked_files.lock().unwrap().par_drain(..));

            if !files_to_parse.is_empty() {
                return false;
            }
        }
        true
    }
    pub fn get_current_depth(&self) -> usize {
        *self.current_depth.read().unwrap()
    }
}

/// This stores the state for the recursively including linked files when
/// using the `--start-at` option with a `--link-depth` greater than 0.
/// Note the paths need to be canonicalized due to canonicalized being used to
/// resolve relative paths outside of start_at
pub struct RecursiveResolver {
    /// the canonicalized root of the vault
    root: PathBuf,
    /// the canonicalized path to start at
    start_at: PathBuf,
    destination: PathBuf,
    //the shared state between this and the caller
    //used to tell caller when to stop recursing
    shared_state: Arc<SharedResolverState>,
}

impl<'a: 'url, 'url> RecursiveResolver {
    pub fn new(
        root: PathBuf,
        start_at: PathBuf,
        destination: PathBuf,
        shared_state: Arc<SharedResolverState>,
    ) -> RecursiveResolver {
        let root = root.canonicalize().unwrap();
        let start_at = start_at.canonicalize().unwrap();
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
    /// postprocess function for recursively resolving links to files outside of start_at
    /// If this is the first iteration, links to files outside of start_at are changed so
    /// that they are to in the root of the destination
    /// if this is any other iteration, links to files outside of start_at are changed so
    /// they strip the difference between root and start_at
    pub fn postprocess(
        &self,
        context: &'a mut Context,
        events: &'url mut MarkdownEvents,
    ) -> PostprocessorResult {
        match *self.shared_state.current_depth.read().unwrap() == 0 {
            true => self.first_run(context, events),
            false => {
                //files to parse should contain only files that have
                //not been parsed in a previous iteration
                if !self
                    .shared_state
                    .files_to_parse
                    .read()
                    .unwrap()
                    .contains(&context.current_file().canonicalize().unwrap())
                {
                    return PostprocessorResult::StopAndSkipNote;
                }
                self.other_runs(context, events)
            }
        }
    }

    ///first run of the postprocessor, changes links to files outside of start_at
    /// and aggregates the filepaths to export in the next iteration
    fn first_run(
        &self,
        _context: &'a mut Context,
        events: &'url mut MarkdownEvents,
    ) -> PostprocessorResult {
        //let path_changed = context.current_file() != &self.start_at;
        for event in events.iter_mut() {
            if let Event::End(Tag::Link(_, url, _)) = event {
                if url.starts_with("https://") || url.starts_with("http://") {
                    continue;
                }

                let vault_path: PathBuf = get_vault_path(url, self.start_at.as_path());

                // may still be within start_at
                if vault_path.starts_with(&self.start_at) {
                    continue;
                }

                if vault_path.exists() {
                    let vaultless_path = vault_path.strip_prefix(self.root.as_path()).unwrap();

                    set_url(url, vaultless_path.to_path_buf());

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
        _context: &'a mut Context,
        events: &'url mut MarkdownEvents,
    ) -> PostprocessorResult {
        //let path_changed = context.current_file() != self.start_at;
        for event in events.iter_mut() {
            if let Event::End(Tag::Link(_, url, _)) = event {
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
                        //don't need to add to linked_files, because it was parsed in the first iteration
                        continue;
                    }
                    //only add if this is not the last iteration
                    if *self.shared_state.current_depth.read().unwrap() < self.shared_state.depth {
                        //only add if it hasn't been parsed in a previous iteration
                        if !self
                            .shared_state
                            .parsed_files
                            .read()
                            .unwrap()
                            .contains(&vault_path)
                        {
                            self.shared_state
                                .linked_files
                                .lock()
                                .unwrap()
                                .push(vault_path);
                        }
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
    // let _=std::mem::replace(
    //     url,
    //     CowStr::from(
    //         utf8_percent_encode(
    //             &format!("{}", link_destination.to_string_lossy()),
    //             PERCENTENCODE_CHARS,
    //         )
    //         .to_string(),
    //     ),
    // );
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
