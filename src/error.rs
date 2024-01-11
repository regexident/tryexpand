use std::convert::From;

use crate::TRYEXPAND_ENV_KEY;

#[derive(Debug)]
pub(crate) enum Error {
    CargoExpandExecution(String),
    CargoFail,
    CargoMetadata(cargo_toml::Error),
    CargoManifestDir,
    Io(std::io::Error),
    Toml(basic_toml::Error),
    Glob(glob::GlobError),
    GlobPattern(glob::PatternError),
    CargoPkgName,
    UnrecognizedEnv(std::ffi::OsString),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::Error::*;

        match self {
            CargoExpandExecution(e) => write!(f, "Failed to execute cargo command: {}", e),
            CargoFail => write!(f, "cargo reported an error"),
            CargoMetadata(e) => write!(f, "{}", e),
            Io(e) => write!(f, "{}", e),
            Toml(e) => write!(f, "{}", e),
            Glob(e) => write!(f, "{}", e),
            GlobPattern(e) => write!(f, "{}", e),
            CargoManifestDir => write!(f, "could not find 'CARGO_MANIFEST_DIR' env var"),
            CargoPkgName => write!(f, "could not find 'CARGO_PKG_NAME' env var"),
            UnrecognizedEnv(e) => write!(
                f,
                "unrecognized value of {key} env var: \"{}\"",
                e.to_string_lossy(),
                key = TRYEXPAND_ENV_KEY
            ),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<basic_toml::Error> for Error {
    fn from(e: basic_toml::Error) -> Self {
        Error::Toml(e)
    }
}

impl From<glob::GlobError> for Error {
    fn from(e: glob::GlobError) -> Self {
        Error::Glob(e)
    }
}

impl From<glob::PatternError> for Error {
    fn from(e: glob::PatternError) -> Self {
        Error::GlobPattern(e)
    }
}
