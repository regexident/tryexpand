use std::borrow::Cow;

use syn::{punctuated::Punctuated, Item, Meta, Token};

use crate::{
    cargo::{line_is_error, line_is_warning, line_should_be_omitted},
    normalization::utils::{apply_replacements, post_process, project_info_replacements},
    project::Project,
    test::Test,
};

pub(crate) fn stdout<'a>(
    input: Cow<'a, str>,
    _project: &Project,
    _test: &Test,
) -> Option<Cow<'a, str>> {
    let output = strip_prelude(input);

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

fn strip_prelude(input: Cow<str>) -> Cow<str> {
    let mut syntax_tree = match syn::parse_file(&input) {
        Ok(syntax_tree) => syntax_tree,
        Err(_) => {
            return input;
        }
    };

    // Strip the following:
    //
    //     #![feature(prelude_import)]
    //
    syntax_tree.attrs.retain(|attr| {
        if let Meta::List(meta) = &attr.meta {
            if meta.path.is_ident("feature") {
                if let Ok(list) =
                    meta.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                {
                    if list.len() == 1 {
                        if let Meta::Path(inner) = &list.first().unwrap() {
                            if inner.is_ident("prelude_import") {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    });

    // Strip the following:
    //
    //     #[prelude_import]
    //     use std::prelude::$edition::*;
    //
    //     #[macro_use]
    //     extern crate std;
    //
    syntax_tree.items.retain(|item| {
        if let Item::Use(item) = item {
            if let Some(attr) = item.attrs.first() {
                if attr.path().is_ident("prelude_import") && attr.meta.require_path_only().is_ok() {
                    return false;
                }
            }
        }
        if let Item::ExternCrate(item) = item {
            if item.ident == "std" {
                return false;
            }
        }
        true
    });

    Cow::from(prettyplease::unparse(&syntax_tree))
}
