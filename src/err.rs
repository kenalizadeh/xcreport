use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use polars::error::PolarsError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum XCReportError {
    #[error("{0}")]
    FilePath(#[source] FilePathError),
    #[error("{0}")]
    FileIO(#[source] std::io::Error),
    #[error("{0}")]
    DirPath(#[source] DirPathError),
    #[error("{0}")]
    UTF8(#[source] std::string::FromUtf8Error),
    #[error("{0}")]
    CommandExecution(#[source] CommandExecutionError),
    #[error("{0}")]
    Polars(#[source] PolarsError),
    #[error("{0}")]
    Serde(#[source] serde_json::Error)
}

#[derive(ThisError, Debug)]
pub enum CommandExecutionError {
    XCodeBuild(#[source] std::io::Error),
    XCPretty(#[source] std::io::Error),
    XCRun(#[source] std::io::Error),
    NonZeroExit { desc: String }
}

impl Display for CommandExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandExecutionError::XCodeBuild(e) => Debug::fmt(&e, f),
            CommandExecutionError::XCPretty(e) => Debug::fmt(&e, f),
            CommandExecutionError::XCRun(e) => Debug::fmt(&e, f),
            CommandExecutionError::NonZeroExit { desc } => f.write_str(desc.deref())
        }
    }
}

#[derive(ThisError, Debug)]
pub enum DirPathError {
    NotFound
}

impl Display for DirPathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DirPathError::NotFound => f.write_str("Directory does not exist"),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum FilePathError {
    NotFound,
    AlreadyExists,
    InvalidType { extension: String }
}

impl Display for FilePathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FilePathError::NotFound => {
                f.write_str("File does not exist.")
            },
            FilePathError::AlreadyExists => {
                f.write_str("File already exists.")
            },
            FilePathError::InvalidType { extension } => {
                write!(f, "File type: {:?} is invalid", extension)
            }
        }
    }
}
