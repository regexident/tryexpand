use syn::{punctuated::Punctuated, Item, Meta, Token};

use crate::project::Project;

// `cargo expand` does always produce some fixed amount of lines that should be ignored
const STDOUT_SKIP_LINES_COUNT: usize = 5;
const STDERR_SKIP_LINES_COUNT: usize = 1;

pub(crate) fn normalize_stdout_expansion(input: &str, project: &Project) -> String {
    normalize_expansion(input, STDOUT_SKIP_LINES_COUNT, project)
}

pub(crate) fn normalize_stderr_expansion(input: &str, project: &Project) -> String {
    normalize_expansion(input, STDERR_SKIP_LINES_COUNT, project)
}

/// Removes specified number of lines and removes some unnecessary or non-deterministic cargo output
fn normalize_expansion(input: &str, num_lines_to_skip: usize, project: &Project) -> String {
    // These prefixes are non-deterministic and project-dependent
    // These prefixes or the whole line shall be removed
    let project_path_prefix = format!(" --> {}/", project.source_dir.to_string_lossy());
    let proj_name_prefix = format!("    Checking {} v0.0.0", project.name);
    let blocking_prefix = "    Blocking waiting for file lock on package cache";

    let lines = input
        .lines()
        .skip(num_lines_to_skip)
        .filter(|line| !line.starts_with(&proj_name_prefix))
        .map(|line| line.strip_prefix(&project_path_prefix).unwrap_or(line))
        .map(|line| line.strip_prefix(blocking_prefix).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");

    let mut syntax_tree = match syn::parse_file(&lines) {
        Ok(syntax_tree) => syntax_tree,
        Err(_) => return lines,
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

    let lines = prettyplease::unparse(&syntax_tree);
    if cfg!(windows) {
        format!("{}\n\r", lines.trim_end_matches("\n\r"))
    } else {
        format!("{}\n", lines.trim_end_matches('\n'))
    }
}
