use std::path::PathBuf;
use std::io::Cursor;
use std::process::{Command, Stdio};
use clap::Parser;
use polars::prelude::*;

mod fs;
mod cli;
mod err;
mod data;
mod df;

use crate::cli::{Cli, Commands};
use crate::err::{FilePathError, XCReportError};
use crate::err::CommandExecutionError;
use crate::fs::{derived_data_path, get_identifier, full_report_path, xcresult_path};
use crate::data::{SquadData, TargetFile, XCodeBuildReport};


fn main() -> Result<(), XCReportError> {
    let cli = Cli::parse();
    let identifier = get_identifier()?;
    process_command(cli.command(), identifier)?;

    Ok(())
}

fn process_command(command: &Commands, identifier: String) -> Result<(), XCReportError> {
    match command {
        Commands::Run {
            input_file,
            project_path,
            workspace,
            scheme,
            destination,
            output_file
        } => {
            let xcresult_path = xcresult_path(&identifier)?;
            run_tests(project_path, &xcresult_path, workspace, scheme, destination)?;
            let report_path = process_xcresult(&input_file, &xcresult_path, &identifier, output_file)?;
            print_result(&report_path, &identifier)?;
        },
        Commands::Generate { input_file, xcresult_file, output_file } => {
            let report_path = process_xcresult(&input_file, &xcresult_file, &identifier, output_file)?;
            print_result(&report_path, &identifier)?;
        }
    }

    Ok(())
}

fn run_tests(
    project_path: &PathBuf,
    xcresult_path: &PathBuf,
    workspace: &PathBuf,
    scheme: &String,
    destination: &String,
) -> Result<(), XCReportError> {

    let derived_data_path = derived_data_path()?;

    let xcbuild_child = Command::new("xcodebuild")
        .args(&[
            "-workspace",
            &workspace.to_str().unwrap(),
            "-scheme",
            &scheme,
            "-derivedDataPath",
            &derived_data_path.to_str().unwrap(),
            "-resultBundlePath",
            &xcresult_path.to_str().unwrap(),
            "-sdk",
            "iphonesimulator",
            "-destination",
            &destination,
            "-enableCodeCoverage",
            "YES",
            "clean",
            "test",
            "CODE_SIGN_IDENTITY=\"\"",
            "CODE_SIGNING_REQUIRED=NO"
        ])
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| XCReportError::CommandExecution(CommandExecutionError::XCodeBuild(e)))?;

    let xcbuild_stdout = xcbuild_child
        .stdout
        .ok_or(XCReportError::CommandExecution(CommandExecutionError::NonZeroExit { desc: String::from("N/A") }))?;

    let xcp_output_file = PathBuf::from_iter([
        &project_path,
        &PathBuf::from("xcpretty_report.html")
    ]);
    let xcp_command = Command::new("xcpretty")
        .args([
            "--test",
            "--simple",
            "--color",
            "--report",
            "html",
            "--output",
            &xcp_output_file.to_str().unwrap()
        ])
        .current_dir(&project_path)
        .stdin(Stdio::from(xcbuild_stdout))
        .status()
        .map_err(|e| XCReportError::CommandExecution(CommandExecutionError::XCPretty(e)))?;

    if !xcp_command.success() {
        let exit_code = xcp_command
            .code()
            .map(|code| {
                code.to_string()
            })
            .unwrap_or(String::from("N/A"));

        return Err(XCReportError::CommandExecution(CommandExecutionError::NonZeroExit { desc: exit_code }))
    }

    Ok(())
}

fn match_squad_files(squads_data: Vec<SquadData>, report: XCodeBuildReport) -> Vec<TargetFile> {
    // TODO: Move this inefficient logic to polars (if possible)
    let all_files = report.get_all_files();
    let mut report_files: Vec<TargetFile> = vec![];

    for file in all_files {
        let squad_file = squads_data
            .iter()
            .find(|squad_data| file.file_path().contains(squad_data.file_name()));

        if let Some(squad_file) = squad_file {
            let mut file = file.clone();
            file.set_squad_name(squad_file.squad_name().clone());
            report_files.push(file);

            if report_files.len() == squads_data.len() {
                break
            }
        } else {
            report_files.push(file.clone());
        }
    }

    return report_files
}

fn process_xcresult(
    input_file: &PathBuf,
    xcresult_file: &PathBuf,
    identifier: &String,
    output_file: &Option<PathBuf>
) -> Result<PathBuf, XCReportError> {

    let squads_data = parse_squads_file(input_file)?;
    let xcodebuild_report = parse_xcresult_json(xcresult_file)?;
    let report_files = match_squad_files(squads_data, xcodebuild_report);

    let json = serde_json::to_string(&report_files)
        .map_err(|e| XCReportError::Serde(e))?;

    let cursor = Cursor::new(json);
    let df = JsonReader::new(cursor)
        .finish()
        .map_err(|e| XCReportError::Polars(e))?;

    let mut full_report_df = df::process_full_report(df)?;
    df::save_full_report(&mut full_report_df, identifier)?;

    let mut report_df = df::process_report(&full_report_df)?;

    if let Some(report_path) = output_file {
        df::save_report_to_output(&mut report_df, &report_path)?;
        Ok(report_path.clone())
    } else {
        let path = df::save_report_to_default(&mut report_df, identifier)?;
        Ok(path)
    }
}

fn parse_xcresult_json(xcresult_file: &PathBuf) -> Result<XCodeBuildReport, XCReportError> {

    if !&xcresult_file.try_exists().unwrap_or_default() {
        return Err(XCReportError::FilePath(FilePathError::NotFound))
    }

    let xcrun_output = Command::new("xcrun")
        .args([
            "xccov",
            "view",
            "--report",
            "--json",
            &xcresult_file.to_str().unwrap()
        ])
        .output()
        .map_err(|e| XCReportError::CommandExecution(CommandExecutionError::XCRun(e)))?;

    let json_report = String::from_utf8(xcrun_output.stdout)
        .map_err(|e| XCReportError::UTF8(e))?;

    let targets: XCodeBuildReport = serde_json::from_str(&json_report)
        .map_err(|e| XCReportError::Serde(e))?;

    Ok(targets)
}

fn parse_squads_file(filepath: &PathBuf) -> Result<Vec<SquadData>, XCReportError> {
    let mut df = CsvReader::from_path(filepath)
        .map_err(|e| XCReportError::Polars(e))?
        .with_columns(Some(vec!["Squad".into(), "Filepath".into()]))
        .has_header(true)
        .finish()
        .map_err(|e| XCReportError::Polars(e))?;

    let mut bytes: Vec<u8> = vec![];

    JsonWriter::new(&mut bytes)
        .with_json_format(JsonFormat::Json)
        .finish(&mut df)
        .map_err(|e| XCReportError::Polars(e))?;

    let squads_data: Vec<SquadData> = serde_json::from_slice(&bytes[..])
        .map_err(|e| XCReportError::Serde(e))?;

    Ok(squads_data)
}

fn print_result(report_path: &PathBuf, identifier: &String) -> Result<(), XCReportError> {
    let full_report_path = full_report_path(&identifier)?;

    println!("\nYour report is ready at:\n{:?}", report_path);
    println!("\nYour full report is at:\n{:?}", full_report_path);

    Ok(())
}