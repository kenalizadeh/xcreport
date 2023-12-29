use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::process::{ExitCode, Termination};
use polars::error::PolarsError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum XCTestError {
    #[error("FileIOError")]
    FileIO(#[source] std::io::Error),
    #[error("FilePathError")]
    FilePath(#[source] FilePathError),
    #[error("DirPathError")]
    DirPath(#[source] DirPathError),
    #[error("UTF8")]
    UTF8(#[source] std::string::FromUtf8Error),
    #[error("CommandExecutionError")]
    CommandExecution(#[source] CommandExecutionError),
    #[error("Polars error")]
    Polars(#[source] PolarsError),
    #[error("Serde error")]
    Serde(#[source] serde_json::Error),
    #[error("CSVParseError")]
    CSVParse(#[source] CSVParseError)
}

#[derive(ThisError, Debug)]
pub enum CommandExecutionError {
    Tuist(#[source] std::io::Error),
    Cocoapods(#[source] std::io::Error),
    XCodeBuild(#[source] std::io::Error),
    XCPretty(#[source] std::io::Error),
    XCRun(#[source] std::io::Error),
    NonZeroExit { desc: String }
}

impl Display for CommandExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandExecutionError::Tuist(e) => Debug::fmt(&e, f),
            CommandExecutionError::Cocoapods(e) => Debug::fmt(&e, f),
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
    InvalidType { extension: String }
}

impl Display for FilePathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FilePathError::NotFound => {
                f.write_str("File does not exist")
            },
            FilePathError::InvalidType { extension } => {
                write!(f, "{}", format!("File type: {:?} is invalid", extension))
            }
        }
    }
}

#[derive(ThisError, Debug)]
pub enum CSVParseError {
    ColumnMissing { name: &'static str }
}

impl Display for CSVParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CSVParseError::ColumnMissing { name } => {
                write!(f, "{}", format!("column {} could not be found", name))
            }
        }
    }
}
