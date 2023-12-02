use regex::Regex;
use std::fmt;

lazy_static! {
    static ref OBSIDIAN_NOTE_LINK_RE: Regex =
        Regex::new(r"^(?P<file>[^#|]+)??(#(?P<section>.+?))??(\|(?P<label>.+?))??$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// ObsidianNoteReference represents the structure of a `[[note]]` or `![[embed]]` reference.
pub struct ObsidianNoteReference<'a> {
    /// The file (note name or partial path) being referenced.
    /// This will be None in the case that the reference is to a section within the same document
    pub file: Option<&'a str>,
    /// If specific, a specific section/heading being referenced.
    pub section: Option<&'a str>,
    /// If specific, the custom label/text which was specified.
    pub label: Option<&'a str>,
}

#[derive(PartialEq, Eq)]
/// RefParserState enumerates all the possible parsing states [RefParser] may enter.
pub enum RefParserState {
    NoState,
    ExpectSecondOpenBracket,
    ExpectRefText,
    ExpectRefTextOrCloseBracket,
    ExpectFinalCloseBracket,
    Resetting,
}

/// RefType indicates whether a note reference is a link (`[[note]]`) or embed (`![[embed]]`).
pub enum RefType {
    Link,
    Embed,
}

/// RefParser holds state which is used to parse Obsidian WikiLinks (`[[note]]`, `![[embed]]`).
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
    pub fn new() -> RefParser {
        RefParser {
            state: RefParserState::NoState,
            ref_type: None,
            ref_text: String::new(),
        }
    }

    pub fn transition(&mut self, new_state: RefParserState) {
        self.state = new_state;
    }

    pub fn reset(&mut self) {
        self.state = RefParserState::NoState;
        self.ref_type = None;
        self.ref_text.clear();
    }
}

impl<'a> ObsidianNoteReference<'a> {
    pub fn from_str(text: &str) -> ObsidianNoteReference {
        let captures = OBSIDIAN_NOTE_LINK_RE
            .captures(text)
            .expect("note link regex didn't match - bad input?");
        let file = captures.name("file").map(|v| v.as_str().trim());
        let label = captures.name("label").map(|v| v.as_str());
        let section = captures.name("section").map(|v| v.as_str().trim());

        ObsidianNoteReference {
            file,
            section,
            label,
        }
    }

    pub fn display(&self) -> String {
        format!("{}", self)
    }
}

impl<'a> fmt::Display for ObsidianNoteReference<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label =
            self.label
                .map(|v| v.to_string())
                .unwrap_or_else(|| match (self.file, self.section) {
                    (Some(file), Some(section)) => format!("{} > {}", file, section),
                    (Some(file), None) => file.to_string(),
                    (None, Some(section)) => section.to_string(),

                    _ => panic!("Reference exists without file or section!"),
                });
        write!(f, "{}", label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_note_refs_from_strings() {
        assert_eq!(
            ObsidianNoteReference::from_str("Just a note"),
            ObsidianNoteReference {
                file: Some("Just a note"),
                label: None,
                section: None,
            }
        );
        assert_eq!(
            ObsidianNoteReference::from_str("A note?"),
            ObsidianNoteReference {
                file: Some("A note?"),
                label: None,
                section: None,
            }
        );
        assert_eq!(
            ObsidianNoteReference::from_str("Note#with heading"),
            ObsidianNoteReference {
                file: Some("Note"),
                label: None,
                section: Some("with heading"),
            }
        );
        assert_eq!(
            ObsidianNoteReference::from_str("Note#Heading|Label"),
            ObsidianNoteReference {
                file: Some("Note"),
                label: Some("Label"),
                section: Some("Heading"),
            }
        );
        assert_eq!(
            ObsidianNoteReference::from_str("#Heading|Label"),
            ObsidianNoteReference {
                file: None,
                label: Some("Label"),
                section: Some("Heading"),
            }
        );
    }

    #[test]
    fn test_display_of_note_refs() {
        assert_eq!(
            "Note",
            ObsidianNoteReference {
                file: Some("Note"),
                label: None,
                section: None,
            }
            .display()
        );
        assert_eq!(
            "Note > Heading",
            ObsidianNoteReference {
                file: Some("Note"),
                label: None,
                section: Some("Heading"),
            }
            .display()
        );
        assert_eq!(
            "Heading",
            ObsidianNoteReference {
                file: None,
                label: None,
                section: Some("Heading"),
            }
            .display()
        );
        assert_eq!(
            "Label",
            ObsidianNoteReference {
                file: Some("Note"),
                label: Some("Label"),
                section: Some("Heading"),
            }
            .display()
        );
        assert_eq!(
            "Label",
            ObsidianNoteReference {
                file: None,
                label: Some("Label"),
                section: Some("Heading"),
            }
            .display()
        );
    }
}
