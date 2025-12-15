use std::borrow::Cow;

use crate::{
    normalization::utils::{
        apply_regex_replacements, apply_replacements, post_process, project_info_replacements,
    },
    options::{FilterTarget, RegexFilter},
    project::Project,
    test::Test,
};

pub(crate) fn stdout<'a>(
    input: Cow<'a, str>,
    _project: &Project,
    _test: &Test,
    filters: &[RegexFilter],
) -> Option<Cow<'a, str>> {
    let output = input;

    let output = apply_regex_replacements(output, filters, FilterTarget::Stdout);

    post_process(output)
}

pub(crate) fn stderr<'a>(
    input: Cow<'a, str>,
    project: &Project,
    test: &Test,
    filters: &[RegexFilter],
) -> Option<Cow<'a, str>> {
    let replacements = project_info_replacements(project, test);

    let trimmed_input = input.trim();

    let output = trimmed_input
        .lines()
        .map(|line| {
            let replacements = replacements.iter().map(|(p, r)| (p.as_str(), r.as_str()));
            apply_replacements(Cow::from(line), replacements)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let output = apply_regex_replacements(Cow::from(output), filters, FilterTarget::Stderr);

    post_process(output)
}
