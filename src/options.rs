use std::collections::HashMap;

/// Options for passing to `cargo expand`/`cargo check`.
#[derive(Clone, Default, Debug)]
pub struct Options {
    // Additional arguments to pass to `cargo expand`/`cargo check`.
    pub args: Vec<String>,
    // Additional env variables to pass to `cargo expand`/`cargo check`.
    pub env: HashMap<String, String>,
    // Whether to skip snapshot writing when running with `TRYEXPAND=overwrite`.
    pub skip_overwrite: bool,
}

impl Options {
    // Appends additional arguments to `self.args`.
    pub fn args<I, T>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        self.args = Vec::from_iter(args.into_iter().map(|arg| arg.as_ref().to_owned()));
        self
    }

    // Appends additional key-value pairs to `self.env`.
    pub fn env<I, K, V>(mut self, env: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.env = HashMap::from_iter(env.into_iter().map(|(key, value)| {
            let key = key.as_ref().to_owned();
            let value = value.as_ref().to_owned();
            (key, value)
        }));
        self
    }

    pub fn skip_overwrite(mut self) -> Self {
        self.skip_overwrite = true;
        self
    }
}
