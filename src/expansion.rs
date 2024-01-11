use std::borrow::Cow;

use syn::{punctuated::Punctuated, Item, Meta, Token};

use crate::project::Project;

pub(crate) fn normalize_stdout_expansion(input: &str) -> String {
    let lines = input.lines().collect::<Vec<_>>().join("\n");

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

    format!("{}\n", lines.trim_end_matches('\n'))
}

pub(crate) fn normalize_stderr_expansion(input: &str, project: &Project) -> String {
    let manifest_dir_path = project.manifest_dir.to_string_lossy();

    let lines = input
        .lines()
        .skip_while(|line| !line.starts_with("error: "))
        .map(|line| {
            if line.starts_with(" --> ") {
                Cow::from(line.replace(manifest_dir_path.as_ref(), ""))
            } else {
                Cow::from(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{}\n", lines.trim_end_matches('\n'))
}
