use std::ffi::OsStr;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use crate::err::{FilePathError, XCTestError};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands
}

impl Cli {
    pub fn command(&self) -> &Commands {
        &self.command
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run tests and generate coverage report
    Run {
        /// Input csv file to match the test results (Squad and Filepath fields required).
        #[arg(short, long, value_parser = parse_input_file)]
        input_file: PathBuf,
        /// Path to your xcode project root.
        #[arg(short, long)]
        project_path: PathBuf,
        /// Xcodebuild argument - Your workspace name.
        #[arg(short, long)]
        workspace: PathBuf,
        /// Xcodebuild argument - Your scheme name.
        #[arg(short, long)]
        scheme: String,
        /// Xcodebuild argument - Simulator destination.
        #[arg(short, long)]
        destination: String,
        /// Optional | File path to save the generated report.
        #[arg(short, long, value_parser = parse_output_file)]
        output_file: Option<PathBuf>
    },
    /// Generate coverage report from test result
    Generate {
        /// Input csv file to match the test results (Squad and Filepath fields required).
        #[arg(short, long, value_parser = parse_input_file)]
        input_file: PathBuf,
        /// Path to the .xcresult file.
        #[arg(short, long, value_parser = parse_xcresult_file)]
        xcresult_file: PathBuf,
        /// Optional | File path to save the generated report.
        #[arg(short, long, value_parser = parse_output_file)]
        output_file: Option<PathBuf>
    }
}

fn parse_file(arg: &str, extension: &str) -> Result<PathBuf, XCTestError> {
    let path = PathBuf::from(arg);
    let path_exists = path.try_exists().unwrap_or_default();

    if !path_exists {
        return Err(XCTestError::FilePath(FilePathError::NotFound))
    }

    if path.extension() != Some(OsStr::new(extension)) {
        let extension = path.extension()
            .unwrap_or(OsStr::new("N/A"))
            .to_os_string()
            .into_string()
            .unwrap_or(String::from("N/A"));

        return Err(XCTestError::FilePath(FilePathError::InvalidType { extension }))
    }

    Ok(path)
}

fn parse_xcresult_file(arg: &str) -> Result<PathBuf, XCTestError> {
    parse_file(arg, "xcresult")
}

fn parse_input_file(arg: &str) -> Result<PathBuf, XCTestError> {
    parse_file(arg, "csv")
}

fn parse_output_file(arg: &str) -> Result<PathBuf, XCTestError> {
    let path = PathBuf::from(arg);
    let path_exists = path.try_exists().unwrap_or_default();

    if path_exists {
        return Err(XCTestError::FilePath(FilePathError::AlreadyExists))
        // return Err(XCTestError::FileAlreadyExists)
    }

    Ok(path)
}