use std::collections::HashMap;

use regex::Regex;

/// Target output stream for a filter
#[derive(Clone, Debug)]
pub(crate) enum FilterTarget {
    Stdout,
    Stderr,
}

/// A compiled regex filter with its replacement string
#[derive(Clone, Debug)]
pub(crate) struct RegexFilter {
    pub target: FilterTarget,
    pub pattern: Regex,
    pub replacement: String,
}

/// Options for passing to `cargo expand`/`cargo check`.
#[derive(Clone, Default, Debug)]
pub(crate) struct Options {
    // Additional arguments to pass to `cargo expand`/`cargo check`.
    pub args: Vec<String>,
    // Additional env variables to pass to `cargo expand`/`cargo check`.
    pub envs: HashMap<String, String>,
    // Whether to skip snapshot writing when running with `TRYEXPAND=overwrite`.
    pub skip_overwrite: bool,
    // Regex filters to apply to output before snapshot comparison.
    pub filters: Vec<RegexFilter>,
}
