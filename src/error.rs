use std::fmt;

/// Unified error kind for text encoding and formatting.
///
/// The encoder ([`crate::encode`]) collects these as recoverable warnings
/// wrapped in [`crate::encode::ErrorFormat`] with positional metadata; the
/// formatter ([`crate::format`]) returns them directly as hard errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    UnknownAlias(String),
    UnmatchedBracket,
    InvalidHexEscape(String),
    IncompleteHexEscape,
    UnknownEscape(String),
    IncompleteEscapeAtEnd,
    UnmatchedBrace,
    EmptyCommand,
    InvalidCommandFormat(String),
    UnknownCommandName {
        name: String,
        code: u16,
    },
    UnknownCharInTrainerName(char),
    UnknownCharacter(char),
    /// A single word is wider than the maximum line width, so it cannot be
    /// wrapped onto any line.
    #[allow(dead_code)] // constructed by rotom's `format`, which chatot does not use
    WordTooLong {
        word: String,
        width: u32,
        max: u32,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::UnknownAlias(alias) => {
                write!(f, "unknown alias '{alias}'. Inserting null code.")
            }
            FormatError::UnmatchedBracket => {
                write!(f, "unmatched '[' in text. Inserting null code.")
            }
            FormatError::InvalidHexEscape(hex) => {
                write!(
                    f,
                    "invalid escape sequence '\\x{hex}'. Inserting null code."
                )
            }
            FormatError::IncompleteHexEscape => {
                write!(f, "incomplete hex escape sequence. Inserting null code.")
            }
            FormatError::UnknownEscape(seq) => {
                write!(f, "unknown escape sequence '{seq}'. Inserting null code.")
            }
            FormatError::IncompleteEscapeAtEnd => write!(
                f,
                "incomplete escape sequence at end of text. Inserting null code."
            ),
            FormatError::UnmatchedBrace => {
                write!(f, "unmatched '{{' in text. Inserting null code.")
            }
            FormatError::EmptyCommand => write!(f, "empty command '{{}}'. Inserting null code."),
            FormatError::InvalidCommandFormat(cmd) => {
                write!(f, "invalid command format '{cmd}'. Inserting null code.")
            }
            FormatError::UnknownCommandName { name, code } => {
                write!(f, "unknown command name '{name}'. Using code 0x{code:04X}.")
            }
            FormatError::UnknownCharInTrainerName(ch) => {
                write!(
                    f,
                    "unknown character '{ch}' in trainer name. Using null code."
                )
            }
            FormatError::UnknownCharacter(ch) => {
                write!(f, "unknown character '{ch}'. Inserting null code.")
            }
            FormatError::WordTooLong { word, width, max } => write!(
                f,
                "word '{word}' is {width}px wide, exceeding the maximum line width of {max}px and cannot be wrapped."
            ),
        }
    }
}

impl std::error::Error for FormatError {}
