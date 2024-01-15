use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub(crate) enum Error {
    #[error("failed to execute cargo command: {0}")]
    CargoExpandExecution(String),
    #[error("cargo reported an error")]
    CargoFail,
    #[error(transparent)]
    CargoToml(#[from] cargo_toml::Error),
    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error("could not find package")]
    CargoPackageNotFound,
    #[error("unsupported rust edition: {edition:?}")]
    UnsupportedRustEdition { edition: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] basic_toml::Error),
    #[error(transparent)]
    Glob(#[from] glob::GlobError),
    #[error(transparent)]
    GlobPattern(#[from] glob::PatternError),
    #[error("could not find 'CARGO_PKG_NAME' env var")]
    CargoPkgName,
    #[error("unrecognized environment variable value: '{key}={value}'")]
    UnrecognizedEnv { key: String, value: String },
    #[error("unexpectedly encountered an empty stdout log for a successful test")]
    UnexpectedEmptyStdOut,
    #[error("unexpectedly encountered an empty stderr log for an unsuccessful test")]
    UnexpectedEmptyStdErr,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
