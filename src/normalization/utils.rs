use std::borrow::Cow;

use crate::{
    options::{FilterTarget, RegexFilter},
    project::Project,
    test::Test,
};

pub(super) fn ensure_trailing_newline(input: Cow<str>) -> Cow<str> {
    if input.ends_with('\n') {
        input
    } else {
        let mut output = input.into_owned();
        output.push('\n');
        Cow::from(output)
    }
}

pub(super) fn is_empty_or_only_whitespace(string: &str) -> bool {
    if string.is_empty() {
        return true;
    }

    string.chars().all(|c| c.is_whitespace())
}

// Replaces string with `None`` if it is either empty or contains only whitespace
// and ensures a trailing new-line otherwise.
pub(super) fn post_process(input: Cow<str>) -> Option<Cow<str>> {
    if is_empty_or_only_whitespace(&input) {
        None
    } else {
        Some(ensure_trailing_newline(input))
    }
}

pub(super) fn project_info_replacements(project: &Project, test: &Test) -> [(String, String); 3] {
    let bin = test.bin.clone();
    let name = project.name.clone();
    let src_path = project.manifest_dir.to_string_lossy().into_owned();
    [
        (bin, "<BIN>".to_owned()),
        (name, "<CRATE>".to_owned()),
        (src_path, "".to_owned()),
    ]
}

pub(super) fn apply_replacements<'a, 'b>(
    string: Cow<'a, str>,
    replacements: impl IntoIterator<Item = (&'b str, &'b str)>,
) -> Cow<'a, str> {
    replacements
        .into_iter()
        .fold(string, |string, (pattern, replacement)| {
            if string.contains(pattern) {
                Cow::from(string.replace(pattern, replacement))
            } else {
                string
            }
        })
}

/// Apply regex filters that target a specific output stream
pub(super) fn apply_regex_replacements<'a>(
    string: Cow<'a, str>,
    filters: &[RegexFilter],
    target: FilterTarget,
) -> Cow<'a, str> {
    filters
        .iter()
        .filter(|f| {
            matches!(
                (&f.target, &target),
                (FilterTarget::Stdout, FilterTarget::Stdout)
                    | (FilterTarget::Stderr, FilterTarget::Stderr)
            )
        })
        .fold(string, |string, filter| {
            Cow::from(
                filter
                    .pattern
                    .replace_all(&string, filter.replacement.as_str())
                    .into_owned(),
            )
        })
}
