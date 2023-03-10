#[cfg(test)]
mod tests;

use crate::error::Error;

use super::{
    replace::replace_classes,
    Classes,
    Message::{self, *},
    Mode, RawRule, TestDraft,
};

/// Minifies the fields of a `Draft`
///
/// If `with_tests` is true, the minified string will include tests
pub(super) fn minify(
    mode: Mode,
    classes: &Classes,
    rules: &[RawRule],
    messages: &[Message<TestDraft>],
    with_tests: bool,
) -> Result<String, Error> {
    let (positive, negative) = minify_tests(messages);

    // Include mode and rules
    let mut minified = format!(
        "~{mode};{rules}",
        mode = mode.as_str(),
        rules = minify_rules(rules, classes)?.join(";"),
    );

    // If tests are enabled
    if with_tests {
        // Add tests, if each intent is not empty
        if !positive.is_empty() {
            minified += &format!(";?+{}", positive.join(" "));
        }
        if !negative.is_empty() {
            minified += &format!(";?!{}", negative.join(" "));
        }
    }

    Ok(minified)
}

/// Minify tests, separate positive and negative intents
fn minify_tests(messages: &[Message<TestDraft>]) -> (Vec<&str>, Vec<&str>) {
    let mut positive = Vec::new();
    let mut negative = Vec::new();

    for msg in messages {
        if let Test(TestDraft { word, intent }) = msg {
            if *intent {
                positive.push(word.as_str());
            } else {
                negative.push(word.as_str());
            }
        }
    }

    (positive, negative)
}

/// Minify raw rules as list of strings
fn minify_rules(rules: &[RawRule], classes: &Classes) -> Result<Vec<String>, Error> {
    let mut strings = Vec::new();

    for RawRule {
        intent,
        pattern,
        line,
        ..
    } in rules
    {
        strings.push(format!(
            "{}{}",
            if *intent { '+' } else { '!' },
            replace_classes(pattern, classes, *line)?
        ));
    }

    Ok(strings)
}
