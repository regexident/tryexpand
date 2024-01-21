use std::borrow::Cow;

use crate::{
    normalization::utils::{apply_replacements, post_process, project_info_replacements},
    project::Project,
    test::Test,
};

pub(crate) fn stdout<'a>(
    input: Cow<'a, str>,
    _project: &Project,
    _test: &Test,
) -> Option<Cow<'a, str>> {
    let output = input;
    post_process(output)
}

pub(crate) fn stderr<'a>(
    input: Cow<'a, str>,
    project: &Project,
    test: &Test,
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

    post_process(Cow::from(output))
}
