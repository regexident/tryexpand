use std::borrow::Cow;

use syn::{punctuated::Punctuated, Item, Meta, Token};

use crate::{project::Project, test::Test};

pub(crate) fn success_stdout(input: String, _project: &Project, _test: &Test) -> Option<String> {
    let mut syntax_tree = match syn::parse_file(&input) {
        Ok(syntax_tree) => syntax_tree,
        Err(_) => {
            return post_process(input);
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

    let output = prettyplease::unparse(&syntax_tree);

    post_process(output)
}

pub(crate) fn success_stderr(input: String, _project: &Project, _test: &Test) -> Option<String> {
    let output = input.trim().lines().collect::<Vec<_>>().join("\n");

    post_process(output)
}

pub(crate) fn failure_stdout(input: String, _project: &Project, _test: &Test) -> Option<String> {
    let output = input.trim().lines().collect::<Vec<_>>().join("\n");

    post_process(output)
}

pub(crate) fn failure_stderr(input: String, project: &Project, test: &Test) -> Option<String> {
    let replacements = std_err_replacements(project, test);

    let trimmed_input = input.trim();

    let output = trimmed_input
        .lines()
        .skip_while(|line| {
            // Sometimes the `cargo expand` command returns a success status,
            // despite an error having occurred, so we need to look for those:

            // Example:
            // ```
            // ...
            // error[E0433]: failed to resolve: use of undeclared crate or module `pwyxfa`
            // --> /tests/expand/fail/test.rs:1:1
            // |
            // 1 | pwyxfa::skpwbd! {
            // | ^^^^^^ use of undeclared crate or module `pwyxfa`
            // For more information about this error, try `rustc --explain E0433`.
            // error: could not compile `<CRATE>` (bin "<BIN>") due to previous error
            // ...
            // ```
            if line.starts_with("error[") {
                return false;
            }

            // ```
            // error: expected item, found `1234`
            // --> /tests/expand/fail/test.rs:1:1
            // |
            // 1 | 1234
            // | ^^^^ expected item
            // |
            // ...
            // ```
            if line.starts_with("error:") {
                return false;
            }

            true
        })
        .map(|line| {
            replacements
                .iter()
                .fold(Cow::from(line), |line, (pattern, replacement)| {
                    if line.contains(pattern) {
                        Cow::from(line.replace(pattern, replacement))
                    } else {
                        line
                    }
                })
        })
        .collect::<Vec<_>>()
        .join("\n");

    post_process(output)
}

fn std_err_replacements(project: &Project, test: &Test) -> [(String, String); 3] {
    let bin = test.bin.clone();
    let name = project.name.clone();
    let src_path = project.manifest_dir.to_string_lossy().into_owned();
    [
        (bin, "<BIN>".to_owned()),
        (name, "<CRATE>".to_owned()),
        (src_path, "".to_owned()),
    ]
}

// Replaces string with `None`` if it is either empty or contains only whitespace
// and ensures a trailing new-line otherwise.
fn post_process(input: String) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    let is_only_whitespace = input
        .lines()
        .all(|line| line.chars().all(|c| c.is_whitespace()));

    if is_only_whitespace {
        return None;
    }

    let mut output = input;

    if !output.ends_with('\n') {
        output.push('\n');
    }

    Some(output)
}
