use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

use regex::Regex;
use snafu::Snafu;

static OBSIDIAN_NOTE_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<file>[^#|]+)??(#(?P<section>.+?))??(\|(?P<label>.+?))??$").unwrap()
});

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// `ObsidianNoteReference` represents the structure of a `[[note]]` or `![[embed]]` reference.
pub struct ObsidianNoteReference {
    /// The file (note name or partial path) being referenced.
    /// This will be None in the case that the reference is to a section within the same document
    pub file: Option<String>,
    /// If specific, a specific section/heading being referenced.
    pub section: Option<String>,
    /// If specific, the custom label/text which was specified.
    pub label: Option<String>,
}

#[derive(Debug, Snafu)]
#[snafu(display("Malformed note reference: {}", reference_text))]
/// This is the error type returned when a string cannot be parsed into an `ObsidianNoteReference`.
pub struct ParseError {
    reference_text: String,
}

#[derive(PartialEq, Eq)]
/// `RefParserState` enumerates all the possible parsing states [`RefParser`] may enter.
pub enum RefParserState {
    NoState,
    ExpectSecondOpenBracket,
    ExpectRefText,
    ExpectRefTextOrCloseBracket,
    ExpectFinalCloseBracket,
    Resetting,
}

/// `RefType` indicates whether a note reference is a link (`[[note]]`) or embed (`![[embed]]`).
pub enum RefType {
    Link,
    Embed,
}

/// `RefParser` holds state which is used to parse Obsidian `WikiLinks` (`[[note]]`, `![[embed]]`).
pub struct RefParser {
    pub state: RefParserState,
    pub ref_type: Option<RefType>,
    // References sometimes come in through multiple events. One example of this is when notes
    // start with an underscore (_), presumably because this is also the literal which starts
    // italic and bold text.
    //
    // ref_text concatenates the values from these partial events so that there's a fully-formed
    // string to work with by the time the final `]]` is encountered.
    pub ref_text: String,
}

impl RefParser {
    pub const fn new() -> Self {
        Self {
            state: RefParserState::NoState,
            ref_type: None,
            ref_text: String::new(),
        }
    }

    pub const fn transition(&mut self, new_state: RefParserState) {
        self.state = new_state;
    }

    pub fn reset(&mut self) {
        self.state = RefParserState::NoState;
        self.ref_type = None;
        self.ref_text.clear();
    }
}

impl ObsidianNoteReference {
    #[must_use]
    pub fn display(&self) -> String {
        format!("{self}")
    }
}

impl FromStr for ObsidianNoteReference {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let captures = OBSIDIAN_NOTE_LINK_RE
            .captures(text)
            .ok_or_else(|| ParseError {
                reference_text: text.into(),
            })?;

        let file = captures
            .name("file")
            .map(|v| v.as_str().trim())
            .filter(|s| !s.is_empty())
            .map(ToString::to_string);
        let label = captures.name("label").map(|v| v.as_str().to_owned());
        let section = captures
            .name("section")
            .map(|v| v.as_str().trim().to_owned());

        Ok(Self {
            file,
            section,
            label,
        })
    }
}

impl fmt::Display for ObsidianNoteReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = if let Some(label) = &self.label {
            label.clone()
        } else {
            match (&self.file, &self.section) {
                (Some(file), Some(section)) => format!("{file} > {section}"),
                (Some(file), None) => file.clone(),
                (None, Some(section)) => section.clone(),
                _ => return Err(fmt::Error),
            }
        };
        write!(f, "{label}")
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("Just a note", Some("Just a note"), None, None)]
    #[case("A note?", Some("A note?"), None, None)]
    #[case("Note#with heading", Some("Note"), None, Some("with heading"))]
    #[case("Note#Heading|Label", Some("Note"), Some("Label"), Some("Heading"))]
    #[case("#Heading|Label", None, Some("Label"), Some("Heading"))]
    fn parse_note_refs_from_strings(
        #[case] input: &str,
        #[case] expected_file: Option<&str>,
        #[case] expected_label: Option<&str>,
        #[case] expected_section: Option<&str>,
    ) {
        assert_eq!(
            input.parse::<ObsidianNoteReference>().unwrap(),
            ObsidianNoteReference {
                file: expected_file.map(String::from),
                label: expected_label.map(String::from),
                section: expected_section.map(String::from),
            }
        );
    }

    #[rstest]
    #[case(Some("Note"), None, None, "Note")]
    #[case(Some("Note"), None, Some("Heading"), "Note > Heading")]
    #[case(None, None, Some("Heading"), "Heading")]
    #[case(Some("Note"), Some("Label"), Some("Heading"), "Label")]
    #[case(None, Some("Label"), Some("Heading"), "Label")]
    fn test_display_of_note_refs(
        #[case] file: Option<&str>,
        #[case] label: Option<&str>,
        #[case] section: Option<&str>,
        #[case] expected: &str,
    ) {
        assert_eq!(
            expected,
            ObsidianNoteReference {
                file: file.map(String::from),
                section: section.map(String::from),
                label: label.map(String::from),
            }
            .display()
        );
    }

    #[test]
    fn test_display_error_case() {
        use std::fmt::Write;

        let reference = ObsidianNoteReference {
            file: None,
            label: None,
            section: None,
        };

        let mut output = String::new();
        let result = write!(&mut output, "{reference}");

        assert!(
            result.is_err(),
            "Expected fmt::Error for reference with no file, label, or section"
        );
    }
}
