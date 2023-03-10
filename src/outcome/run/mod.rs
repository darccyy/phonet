#[cfg(test)]
mod tests;

use self::Validity::*;
use super::{
    FailKind::*,
    Outcome,
    PassStatus::{self, *},
    TestOutcome,
};
use crate::{
    draft::{Draft, Message::*, Note, Rule, TestDraft},
    REGEX_MATCH_FAIL,
};

/// Whether test *matches* or not
///
/// NOTE: This is different to whether a test *passes* (See `PassStatus`)
#[derive(Debug, PartialEq)]
pub(crate) enum Validity {
    Valid,
    Invalid(Option<Note>),
}

impl Outcome {
    /// Run drafted tests, return `Output`
    pub fn run(draft: &Draft) -> Self {
        // No messages
        if draft.messages.is_empty() {
            return Self {
                messages: Vec::new(),
                fail_count: 0,
            };
        }

        // Builders
        let mut list = Vec::new();
        let mut fail_count = 0;

        // Loop messages
        for msg in &draft.messages {
            list.push(match msg {
                // Move note
                Info(note) => Info(note.clone()),
                // Run test
                Test(test) => {
                    let outcome = run_test(test.clone(), &draft.rules);

                    // Increase fail count if failed
                    if outcome.status.is_fail() {
                        fail_count += 1;
                    }

                    Test(outcome)
                }
            });
        }

        Self {
            messages: list,
            fail_count,
        }
    }
}

/// Run `TestDraft` against rules, return `TestOutcome`
fn run_test(test: TestDraft, rules: &[Rule]) -> TestOutcome {
    // Validate test
    let validity = validate_test(&test.word, rules);

    // Get status
    let status = get_status(validity, test.intent);

    TestOutcome {
        intent: test.intent,
        word: test.word,
        status,
    }
}

/// Check if test is valid against rules
pub(crate) fn validate_test(word: &str, rules: &[Rule]) -> Validity {
    // Check for match with every rule, if not, return fail
    for Rule {
        intent,
        pattern,
        note,
    } in rules
    {
        // Check if rule matches, and whether match signifies returning invalid or continuing
        if intent ^ pattern.is_match(word).expect(REGEX_MATCH_FAIL) {
            return Invalid(note.clone());
        }
    }

    Valid
}

/// Get `PassStatus` from `Validity` and test `intent`
fn get_status(validity: Validity, intent: bool) -> PassStatus {
    // Check if validity status matches test intent
    let is_pass = !(matches!(validity, Valid) ^ intent);

    // Get status from validity
    if is_pass {
        // Test passes
        Pass
    } else {
        // Test fails
        Fail(match validity {
            // Test was valid, but should have been invalid
            Valid => ShouldBeInvalid,

            // Test was invalid, but should have been valid
            Invalid(reason) => match reason {
                // Custom reason
                Some(reason) => CustomReason(reason),
                // No reason was given
                None => NoReasonGiven,
            },
        })
    }
}
