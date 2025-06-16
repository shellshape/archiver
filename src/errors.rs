use std::{io, time};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed reading source dir: {0}")]
    ReadingSourceDir(io::Error),

    #[error("failed getting file metadata: {0}")]
    Metadata(io::Error),

    #[error("failed getting mtime: {0}")]
    ModifiedTime(io::Error),

    #[error("failed getting mtime time since unix epoch: {0}")]
    ModifiedTimeDuration(time::SystemTimeError),

    #[error("could not get mtime as chrono::DateTime")]
    ModifiedTimeToChrono,

    #[error("failed creating target directory: {0}")]
    CreatingTargetDirectory(anyhow::Error),

    #[error("failed opening source file: {0}")]
    OpeningSourceFile(io::Error),

    #[error("failed creating target file: {0}")]
    CreatingTargetFile(io::Error),

    #[error("failed copying data: {0}")]
    CopyingFileContents(io::Error),

    #[error("failed deleting source file: {0}")]
    DeletingSourceFile(io::Error),
}
