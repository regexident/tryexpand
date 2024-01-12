use std::borrow::Cow;

use syn::{punctuated::Punctuated, Item, Meta, Token};

use crate::{project::Project, test::Test};

pub(crate) fn success_stdout(input: String, _project: &Project, _test: &Test) -> String {
    let mut syntax_tree = match syn::parse_file(&input) {
        Ok(syntax_tree) => syntax_tree,
        Err(_) => return input.trim().to_owned(),
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

    format!("{}\n", lines.trim_end_matches('\n'))
}

pub(crate) fn failure_stdout(input: String, _project: &Project, _test: &Test) -> String {
    input.trim().to_owned()
}

pub(crate) fn failure_stderr(input: String, project: &Project, test: &Test) -> String {
    let replacements = std_err_replacements(project, test);
    let mut has_errors = false;

    let mut output = input
        .trim()
        .lines()
        .skip_while(|line| {
            // Sometimes the `cargo expand` command returns a success status,
            // despite an error having occurred, so we need to look for those:
            has_errors |= line.starts_with("error: ");

            !has_errors
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

    output.push('\n');
    output
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
