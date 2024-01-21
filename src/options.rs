use std::collections::HashMap;

/// Options for passing to `cargo expand`/`cargo check`.
#[derive(Clone, Default, Debug)]
pub struct Options {
    // Additional arguments to pass to `cargo expand`/`cargo check`.
    pub args: Vec<String>,
    // Additional env variables to pass to `cargo expand`/`cargo check`.
    pub envs: HashMap<String, String>,
    // Whether to skip snapshot writing when running with `TRYEXPAND=overwrite`.
    pub skip_overwrite: bool,
}
