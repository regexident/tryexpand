use std::path::PathBuf;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub(crate) enum Error {
    #[error("failed to execute cargo command: {0}")]
    CargoExpandExecution(String),
    #[error("stdout unavailable")]
    StdOutUnavailable,
    #[error("could not load cargo metadata from {directory:?}")]
    CargoMetadata {
        directory: PathBuf,
        source: cargo_metadata::Error,
    },
    #[error("could not find package")]
    CargoPackageNotFound,
    #[error("unsupported rust edition: {edition:?}")]
    UnsupportedRustEdition { edition: String },
    #[error("could not remove directory: {path:?}")]
    RemovingDirectoryFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not create directory: {path:?}")]
    CreatingDirectoryFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not read from file: {path:?}")]
    ReadingFileFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not write to file: {path:?}")]
    WritingFileFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not spawn process: {0:?}")]
    SpawningProcessFailed(#[source] std::io::Error),
    #[error("could not serialize cargo manifest to toml")]
    CargoManifestSerializationFailed(#[source] basic_toml::Error),
    #[error("could not serialize cargo config to toml")]
    CargoConfigSerializationFailed(#[source] basic_toml::Error),
    #[error("could not access glob path")]
    Glob(#[from] glob::GlobError),
    #[error("could not parse glob path")]
    GlobPattern(#[from] glob::PatternError),
    #[error("could not find 'CARGO_PKG_NAME' env var")]
    CargoPkgName,
    #[error("unrecognized environment variable value: '{key}={value}'")]
    UnrecognizedEnv { key: String, value: String },
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
