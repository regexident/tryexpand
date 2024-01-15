use std::collections::HashMap;

#[derive(Clone, Default, Debug)]
pub struct Options {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

impl Options {
    pub fn args<I, T>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        self.args = Vec::from_iter(args.into_iter().map(|arg| arg.as_ref().to_owned()));
        self
    }

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
}
