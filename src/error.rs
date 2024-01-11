use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub(crate) enum Error {
    #[error("failed to execute cargo command: {0}")]
    CargoExpandExecution(String),
    #[error("cargo reported an error")]
    CargoFail,
    #[error(transparent)]
    CargoMetadata(#[from] cargo_toml::Error),
    #[error("could not find 'CARGO_MANIFEST_DIR' env var")]
    CargoManifestDir,
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
    #[error("unrecognized value of 'TRYEXPAND_ENV_KEY' env var: '{0}'")]
    UnrecognizedEnv(String),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
