use std::borrow::Cow;

use crate::{
    cargo::{line_is_error, line_is_warning, line_should_be_omitted},
    normalization::utils::{apply_replacements, post_process, project_info_replacements},
    project::Project,
    test::Test,
};

pub(crate) fn stdout<'a>(
    _input: Cow<'a, str>,
    _project: &Project,
    _test: &Test,
) -> Option<Cow<'a, str>> {
    // The stdout of `cargo check` isn't really that interesting to us.
    None
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
        .skip_while(|line| !line_is_error(line))
        .filter(|line| !line_should_be_omitted(line))
        .filter(|line| !line_is_warning(line))
        .map(|line| {
            let replacements = replacements.iter().map(|(p, r)| (p.as_str(), r.as_str()));
            apply_replacements(Cow::from(line), replacements)
        })
        .collect::<Vec<_>>()
        .join("\n");

    post_process(Cow::from(output))
}
