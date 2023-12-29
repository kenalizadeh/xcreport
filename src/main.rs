use std::error::Error;
use std::path::PathBuf;
use std::fmt::Display;
use std::fs::File;
use std::io::{Cursor, Read};
use std::ops::{Div, Mul};
use std::process::{Command, Stdio};
use clap::Parser;
use polars::prelude::*;

mod fs;
mod args;
mod errors;
mod model;

use crate::args::{Cli, Commands};
use crate::errors::{FilePathError, XCTestError};
use crate::errors::CommandExecutionError;
use crate::fs::{home_path, setup_home_dir, xcresult_path};
use crate::model::{SquadData, TargetFile, XCodeBuildReport};


fn main() -> Result<(), XCTestError> {

    let cli = Cli::parse();

    setup_home_dir()?;

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command() {
        Commands::Run { input_file, project_path, clean } => {
            println!("Command::Run input_file: {:?}, path: {:?}", input_file, project_path);
            let res = run_tests(project_path, *clean);
            match res {
                Ok(_) => {
                    println!("run_tests res ok");
                    let xcresult_file = xcresult_path()?;
                    let res = process_xcresult(&input_file, &xcresult_file);
                    println!("process_xcresult res: {:?}", &res);
                },
                Err(e) => {
                    println!("run_tests error: {:?}", e);
                }
            }
        },
        Commands::Generate { input_file, xcresult_file } => {
            println!("Command::Generate input_file: {:?}", input_file);
            let res = process_xcresult(&input_file, &xcresult_file);
            println!("process_xcresult res: {:?}", &res);
        },
        Commands::ShowReport => {
            println!("Command::ShowReport")
        }
    }

    Ok(())
}

fn run_tests(project_path: &PathBuf, clean: bool) -> Result<(), XCTestError> {

    if clean {
        println!("- Generating project with Tuist...");
        Command::new("tuist")
            .args(["generate", "--no-open", "--no-cache"])
            .stdout(Stdio::null())
            .current_dir(project_path)
            .status()
            .map_err(|e| XCTestError::CommandExecution(CommandExecutionError::Tuist(e)))?;

        println!("- Installing pods...");
        Command::new("pod")
            .args(["install", "--repo-update", "--clean-install"])
            .stdout(Stdio::null())
            .current_dir(project_path)
            .status()
            .map_err(|e| XCTestError::CommandExecution(CommandExecutionError::Cocoapods(e)))?;
    }

    let xctest_home = home_path()?;
    let workspace_file_path = PathBuf::from_iter([
        &project_path,
        &PathBuf::from("IBAMobileBank.xcworkspace")
    ]);
    let derived_data_path = PathBuf::from_iter([
        &xctest_home,
        &PathBuf::from("derived_data")
    ]);
    let xcbuild_child = Command::new("xcodebuild")
        .args(&[
            "-workspace",
            workspace_file_path.to_str().unwrap(),
            "-scheme",
            "IBAMobileBank-Test",
            "-derivedDataPath",
            &derived_data_path.to_str().unwrap(),
            "-resultBundlePath",
            &xcresult_path()?.to_str().unwrap(),
            "-sdk",
            "iphonesimulator",
            "-destination",
            "platform=iOS Simulator,name=iPhone 14,OS=17.0.1",
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
        .map_err(|e| XCTestError::CommandExecution(CommandExecutionError::XCodeBuild(e)))?;

    let xcbuild_stdout = xcbuild_child
        .stdout
        .ok_or(XCTestError::CommandExecution(CommandExecutionError::NonZeroExit { desc: String::from("N/A") }))?;

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
        .map_err(|e| XCTestError::CommandExecution(CommandExecutionError::XCPretty(e)))?;

    if !xcp_command.success() {
        let exit_code = xcp_command
            .code()
            .map(|code| {
                code.to_string()
            })
            .unwrap_or(String::from("N/A"));

        return Err(XCTestError::CommandExecution(CommandExecutionError::NonZeroExit { desc: exit_code }))
    }

    Ok(())
}

fn process_xcresult(input_file: &PathBuf, xcresult_file: &PathBuf) -> Result<DataFrame, XCTestError> {

    let squads_data = load_squads_file(input_file)?;
    let xcodebuild_report = parse_xcresult_json(xcresult_file)?;

    let all_files = xcodebuild_report.get_all_files();

    // TODO: Move this logic to polars
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

    let json = serde_json::to_string(&report_files)
        .map_err(|e| XCTestError::Serde(e))?;

    let home_path = home_path()?;
    let report_path = PathBuf::from_iter([
        &home_path,
        &PathBuf::from("report.csv")
    ]);
    let raw_report_path = PathBuf::from_iter([
        &home_path,
        &PathBuf::from("raw_report.csv")
    ]);

    let cursor = Cursor::new(json);

    // Raw Report
    let mut df = JsonReader::new(cursor)
        .finish()
        .map_err(|e| XCTestError::Polars(e))?
        .lazy()
        .sort_by_exprs(
            vec![col("squad_name")],
            vec![false],
            true,
            true
        )
        .rename(
            [
                "path",
                "covered_lines",
                "executable_lines",
                "line_coverage",
                "squad_name"
            ],
            [
                "Filepath",
                "Covered Lines",
                "Executable Lines",
                "Line Coverage",
                "Squad"
            ],
        )
        .with_column(
            col("Squad")
                .fill_null(Expr::Literal(LiteralValue::Utf8(String::from("N/A"))))
        )
        .collect()
        .map_err(|e| XCTestError::Polars(e))?;

    let mut file = File::create(&raw_report_path)
        .map_err(|e| XCTestError::FileIO(e))?;

    CsvWriter::new(&mut file)
        .finish(&mut df)
        .map_err(|e| XCTestError::Polars(e))?;

    // Report
    let mut df = process_report(df)?;

    let mut file = File::create(&report_path)
        .map_err(|e| XCTestError::FileIO(e))?;

    CsvWriter::new(&mut file)
        .finish(&mut df)
        .map_err(|e| XCTestError::Polars(e))?;

    Ok(df)
}

fn process_report(report: DataFrame) -> Result<DataFrame, XCTestError> {
    let frame = report
        .lazy()
        .group_by(["Squad"])
        .agg([
            count(),
            col("Covered Lines").sum(),
            col("Executable Lines").sum()
        ])
        .with_column(
            col("Covered Lines")
                .cast(DataType::Float64)
                .div(col("Executable Lines"))
                .mul(Expr::Literal(LiteralValue::Float64(100_f64)))
                .round(2)
                .alias("Coverage %")
        )
        .sort_by_exprs(
            vec![col("Squad")],
            vec![false],
            true,
            true
        )
        .with_column(
            col("Squad")
                .fill_null(Expr::Literal(LiteralValue::Utf8(String::from("N/A"))))
        )
        .rename(["count"], ["Count"])
        .collect()
        .map_err(|e| XCTestError::Polars(e))?;

    Ok(frame)
}

fn parse_xcresult_json(xcresult_file: &PathBuf) -> Result<XCodeBuildReport, XCTestError> {

    if !&xcresult_file.try_exists().unwrap_or_default() {
        return Err(XCTestError::FilePath(FilePathError::NotFound))
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
        .map_err(|e| XCTestError::CommandExecution(CommandExecutionError::XCRun(e)))?;

    let json_report = String::from_utf8(xcrun_output.stdout)
        .map_err(|e| XCTestError::UTF8(e))?;

    let targets: XCodeBuildReport = serde_json::from_str(&json_report)
        .map_err(|e| XCTestError::Serde(e))?;

    Ok(targets)
}

fn load_squads_file(filepath: &PathBuf) -> Result<Vec<SquadData>, XCTestError> {
    let mut df = CsvReader::from_path(filepath)
        .map_err(|e| XCTestError::Polars(e))?
        .with_columns(Some(vec!["Squad".into(), "Filepath".into()]))
        .has_header(true)
        .finish()
        .map_err(|e| XCTestError::Polars(e))?;

    let mut bytes: Vec<u8> = vec![];

    JsonWriter::new(&mut bytes)
        .with_json_format(JsonFormat::Json)
        .finish(&mut df)
        .unwrap();

    let squads_data: Vec<SquadData> = serde_json::from_slice(&bytes[..]).unwrap();

    Ok(squads_data)
}
