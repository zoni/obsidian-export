//! A collection of officially maintained [postprocessors][crate::Postprocessor].

use super::{Context, PostprocessorResult};
use regex::Regex;

const IGNORE_REGEX_0: &str = r"%% EXPORT_IGNORE_BEGIN %%(.|\n)*?%% EXPORT_IGNORE_END %%";
const IGNORE_REGEX_1: &str = r"# EXPORT_IGNORE_BEGIN(.|\n)*?# EXPORT_IGNORE_END";

/// This postprocessor removes all
pub fn remove_ignore_blocks(
    _context: &mut Context,
    string: &mut String,
) -> PostprocessorResult {
    let re = Regex::new(IGNORE_REGEX_0).unwrap();
    *string = re.replace_all(string, "").to_string();
     
    let re = Regex::new(IGNORE_REGEX_1).unwrap();
    *string = re.replace_all(string, "").to_string();
    PostprocessorResult::Continue
}