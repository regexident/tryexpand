use std::borrow::Cow;

use crate::{
    normalization::utils::{apply_replacements, post_process, project_info_replacements},
    project::Project,
    test::Test,
};

pub(crate) fn stdout<'a>(
    input: Cow<'a, str>,
    project: &Project,
    test: &Test,
) -> Option<Cow<'a, str>> {
    let time_regex = regex::Regex::new(r"; finished in .+$").unwrap();

    let replacements = project_info_replacements(project, test);

    let trimmed_input = input.trim();

    let output = trimmed_input
        .lines()
        .map(|line| {
            // Look for lines of the following pattern and normalize the time as `<TIME>`:
            // test result: ... finished in 0.03s
            if line.trim().starts_with("test result:") {
                time_regex.replace(line, "; finished in <TIME>")
            } else {
                Cow::from(line)
            }
        })
        .map(|line| {
            let replacements = replacements.iter().map(|(p, r)| (p.as_str(), r.as_str()));
            apply_replacements(line, replacements)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

    post_process(Cow::from(output))
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
