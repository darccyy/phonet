/// Minify draft to string
mod minify;
/// Parse functions
mod parse;
/// Substitute class names recursively
mod replace;
/// Split file into statements
mod statements;
/// Holds types for `Draft` struct
mod types;

pub(crate) use self::replace::replace_classes;
pub use self::types::*;

use std::collections::HashMap;

use fancy_regex_macro::regex;

use self::{minify::minify, parse::parse_rules, statements::split_statements};
use crate::{error::Error, outcome::Outcome, REGEX_MATCH_FAIL};

/// Shorthand for `Err(Error::Parse(ParseError::______))`
#[macro_export]
macro_rules! parse_error {
    ( $line: expr, $kind: ident ) => {
            Err(crate::error::Error::Parse(
            crate::error::ParseError::$kind,
            $line,
        ))
    };

    ( $line: expr, $kind: ident, $( $value: expr )* ) => {
        Err(crate::error::Error::Parse(
            crate::error::ParseError::$kind(
                $( $value )*
            ),
            $line,
        ))
    };
}

/// Parsed *Phonet* file
#[derive(Debug, PartialEq)]
pub struct Draft {
    /// List of defined rules
    pub rules: Vec<Rule>,
    /// List of messages to be displayed
    ///
    /// Each item may be a `Note` and `TestDraft`
    pub messages: Vec<Message<TestDraft>>,
    /// Transcription mode of file
    pub mode: Mode,
    /// Amount of tests in `messages` field
    pub test_count: usize,

    pub(crate) raw_rules: Vec<RawRule>,
    pub(crate) raw_classes: Classes,
}

impl Draft {
    /// Run drafted tests
    pub fn run(self) -> Outcome {
        Outcome::run(self)
    }

    /// Parse Phonet `Draft` from file
    pub fn from(file: &str) -> Result<Self, Error> {
        // Split file into statements
        let statements = split_statements(file);

        // Field builders
        let mut messages = Vec::new();
        let mut mode: Option<Mode> = None;

        // Field builders without regex parsed
        let mut raw_rules = Vec::new();
        let mut raw_classes = HashMap::new();

        // Most recent note
        let mut last_note: Option<Note> = None;

        // Loop statements
        for (statement, line) in statements {
            let statement = statement.trim();

            // Skip if blank
            if statement.is_empty() {
                continue;
            }

            // Get line operator first character
            let mut chars = statement.chars();
            let Some(operator) = chars.next() else {
                continue;
            };

            // Match line operator
            match operator {
                // Comment
                '#' => continue,

                // Mode
                '~' => {
                    // Fail if mode is already defined
                    if mode.is_some() {
                        return parse_error!(line, ModeAlreadyDefined);
                    }

                    // Remove spaces
                    while chars.as_str().starts_with(' ') {
                        chars.next();
                    }

                    // Select mode
                    mode = Some(match Mode::from_options(chars.next(), chars.last()) {
                        Some(value) => value,
                        None => return parse_error!(line, InvalidModeSpecifier),
                    });
                }

                // Class
                '$' => {
                    let mut split = chars.as_str().split('=');

                    // Get class name
                    let Some(name) = split.next() else {
                        return parse_error!(line, NoClassName);
                    };
                    let name = name.trim();

                    // Check if name is valid
                    if !regex!(r"^\w+$").is_match(name).expect(REGEX_MATCH_FAIL) {
                        return parse_error!(line, InvalidClassName, name.to_string());
                    }

                    // Check that class name does not exist
                    if raw_classes.get(name).is_some() {
                        return parse_error!(line, ClassAlreadyExists, name.to_string());
                    }

                    // Get class pattern
                    let Some(pattern) = split.next() else {
                        return parse_error!(line, NoClassPattern, name.to_string());
                    };

                    // Add class
                    // Wrap value in NON-CAPTURING GROUP (just in case)
                    // This is non-capturing, for classes to work with back-references
                    // otherwise classes would be inherently capturing, and count towards group index in back-reference
                    raw_classes.insert(name.trim().to_string(), pattern.trim().to_string());
                }

                // Rule
                '+' | '!' => {
                    // `+` for true, `!` for false
                    let intent = operator == '+';

                    let pattern = chars.as_str().replace(' ', "");

                    // Get most recent note, owned
                    let note = last_note.clone();

                    // Add rule
                    raw_rules.push(RawRule {
                        intent,
                        pattern,
                        note,
                    })
                }

                // Test
                '?' => {
                    // Remove spaces
                    while chars.as_str().starts_with(' ') {
                        chars.next();
                    }

                    // Get intent
                    let intent = match chars.next() {
                        // Should be INVALID to pass
                        Some('+') => true,
                        // Should be VALID to pass
                        Some('!') => false,

                        // Unknown or no character
                        _ => {
                            return parse_error!(line, InvalidTestIntent);
                        }
                    };

                    // Split at space
                    for word in chars.as_str().split_whitespace() {
                        let word = word.trim().to_string();

                        // Add test
                        if !word.is_empty() {
                            messages.push(Message::Test(TestDraft { intent, word }));
                        }
                    }
                }

                // Note
                '*' => {
                    let note = chars.as_str().trim();

                    if note.is_empty() {
                        return parse_error!(line, EmptyNote);
                    }

                    // Add message
                    messages.push(Message::Info(Note(note.to_string())));

                    // Add note
                    last_note = Some(Note(note.to_string()));
                }

                // Unknown line operator
                _ => {
                    return parse_error!(line, UnknownStatementOperator, operator);
                }
            }
        }

        // Get amount of tests in messages
        let test_count = messages.iter().filter(|msg| msg.is_test()).count();

        // Use default mode if none specified
        let mode = mode.unwrap_or_default();

        // let minified = minify(mode, &raw_classes, &raw_rules, &messages)?;

        Ok(Self {
            rules: parse_rules(&raw_rules, &raw_classes)?,
            raw_rules,
            messages,
            mode,
            test_count,
            raw_classes,
        })
    }

    /// Returns a minified version of the original file of the `Draft`
    ///
    /// If `with_tests` is true, the minified string will include tests
    pub fn minify(&self, with_tests: bool) -> Result<String, Error> {
        minify(
            self.mode,
            &self.raw_classes,
            &self.raw_rules,
            &self.messages,
            with_tests,
        )
    }
}
