use std::path::Path;

use crate::error::{Error, Result};

pub(crate) fn remove_dir_all<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    std::fs::remove_dir_all(path).map_err(|source| Error::RemovingDirectoryFailed {
        path: path.to_owned(),
        source,
    })
}

pub(crate) fn create_dir_all<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(|source| Error::CreatingDirectoryFailed {
        path: path.to_owned(),
        source,
    })
}

pub(crate) fn read<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    std::fs::read(path).map_err(|source| Error::ReadingFileFailed {
        path: path.to_owned(),
        source,
    })
}

pub(crate) fn write<P, C>(path: P, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let path = path.as_ref();
    std::fs::write(path, contents.as_ref()).map_err(|source| Error::WritingFileFailed {
        path: path.to_owned(),
        source,
    })
}
