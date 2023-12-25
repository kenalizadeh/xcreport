mod mods;

use std::path::PathBuf;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::process::{Command, Stdio};
use std::ops::Deref;
use clap::{Parser, Subcommand};
use polars::prelude::*;

use crate::mods::errors::{CommandExecutionError, CSVParseError, DirPathError, FilePathError, XCTestError};

/// Simple program to greet a person
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Optional name to operate on
    name: Option<String>,

    /// Turn debugging information on
    #[arg(short, long)]
    debug: bool
}

#[derive(Subcommand)]
enum Commands {
    /// Run tests and generate coverage report
    Run {
        #[arg(short, long, value_parser = parse_input_file)]
        input_file: PathBuf,
        #[arg(short, long)]
        project_path: PathBuf,
        #[arg(short, long, default_value_t = false)]
        clean: bool
    },
    /// Generate coverage report from test result
    Generate {
        /// Path to directory
        #[arg(short, long, value_parser = parse_input_file)]
        input_file: PathBuf,
        #[arg(short, long, value_parser = parse_xcresult_file)]
        xcresult_file: PathBuf
    },
    /// Show last produced report
    ShowReport
}

fn main() -> Result<(), XCTestError> {

    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name {
        println!("Value for name: {name}");
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    if cli.debug {
        println!("Debug mode is on")
    } else {
        println!("Debug mode is off")
    }

    setup_home_dir().unwrap();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Run { input_file, project_path, clean } => {
            println!("Command::Run input_file: {:?}, path: {:?}", input_file, project_path);
            let res = run_tests(project_path, *clean, cli.debug);
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

#[derive(Debug)]
struct RunParams {
    input_file: PathBuf,
    project_path: PathBuf,
    clean: bool
}

fn setup_home_dir() -> Result<(), XCTestError> {
    let xctest_home = home_path()?;

    fs::create_dir_all(&xctest_home)
        .map_err(|e| {
            println!("setup_home_dir create error: {:?}", e);
            return XCTestError::FileIOError(e)
        })?;

    Ok(())
}

fn home_path() -> Result<PathBuf, XCTestError> {
    let home_path = home::home_dir().ok_or(XCTestError::DirPathError(DirPathError::NotFound))?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from(".xctest")]))
}

fn xcresult_path() -> Result<PathBuf, XCTestError> {
    let home_path = home_path()?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from("result.xcresult")]))
}

fn run_tests(project_path: &PathBuf, clean: bool, debug: bool) -> Result<(), XCTestError> {

    if clean {
        println!("- Generating project with Tuist...");
        Command::new("tuist")
            .args(["generate", "--no-open", "--no-cache"])
            .stdout(Stdio::null())
            .current_dir(project_path)
            .status()
            .map_err(|e| {
                if debug {
                    println!("error: {:?}", e);
                }
                return XCTestError::CommandExecutionError(CommandExecutionError::Tuist(e))
            })?;

        println!("- Installing pods...");
        Command::new("pod")
            .args(["install", "--repo-update", "--clean-install"])
            .stdout(Stdio::null())
            .current_dir(project_path)
            .status()
            .map_err(|e| {
                if debug {
                    println!("error: {:?}", e);
                }
                return XCTestError::CommandExecutionError(CommandExecutionError::Cocoapods(e))
            })?;
    }

    let xctest_home = home_path()?;
    let workspace_file_path = PathBuf::from_iter([&project_path, &PathBuf::from("IBAMobileBank.xcworkspace")]);
    let derived_data_path = PathBuf::from_iter([&xctest_home, &PathBuf::from("derived_data")]);
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
        .map_err(|e| {
            if debug {
                println!("xcbuild_child error: {:?}", e);
            }
            return XCTestError::CommandExecutionError(CommandExecutionError::XCodeBuild(e))
        })?;

    let xcbuild_stdout = xcbuild_child
        .stdout
        .ok_or(XCTestError::CommandExecutionError(CommandExecutionError::NonZeroExit { desc: String::from("N/A") }))?;

    let xcp_output_file = PathBuf::from_iter([&project_path, &PathBuf::from("xcpretty_report.html")]);
    let xcp_command = Command::new("xcpretty")
        .args(["-t", "-s", "-c", "--report", "html", "--output", &xcp_output_file.to_str().unwrap()])
        .current_dir(&project_path)
        .stdin(Stdio::from(xcbuild_stdout))
        .status()
        .map_err(|e| {
            if debug {
                println!("xcp_command error: {:?}", e);
            }
            return XCTestError::CommandExecutionError(CommandExecutionError::XCPretty(e))
        })?;

    if !xcp_command.success() {
        if debug {
            println!("xcp_command code: {:?}", xcp_command.code());
        }

        let exit_code = xcp_command
            .code()
            .map(|code| {
                code.to_string()
            })
            .unwrap_or(String::from("N/A"));

        return Err(XCTestError::CommandExecutionError(CommandExecutionError::NonZeroExit { desc: exit_code }))
    }

    Ok(())
}

fn process_xcresult(input_file: &PathBuf, xcresult_file: &PathBuf) -> Result<(), XCTestError> {
    parse_xcresult_to_dataframe(xcresult_file)?;

    let df = load_squads_file(input_file)?;



    Ok(())
}

fn parse_xcresult_to_dataframe(xcresult_file: &PathBuf) -> Result<DataFrame, XCTestError> {
    let home_path = home_path()?;
    let raw_report = PathBuf::from_iter(&[home_path, PathBuf::from("raw_report.json")]);

    if !&xcresult_file.try_exists().unwrap_or_default() {
        return Err(XCTestError::FilePathError(FilePathError::NotFound))
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
        .map_err(|e| {
            println!("xcrun error: {:?}", e);
            return XCTestError::CommandExecutionError(CommandExecutionError::XCRun(e))
        })?;

    fs::write(&raw_report, xcrun_output.stdout).map_err(|e| {
        println!("report write err: {:?}", e);
        return XCTestError::FileIOError(e)
    })?;

    let raw_report_file = File::open(&raw_report)
        .map_err(|e| {
            println!("raw_report_file err: {:?}", e);
            return XCTestError::FileIOError(e)
        })?;

    let df = JsonReader::new(raw_report_file)
        .with_json_format(JsonFormat::Json)
        .finish()
        .map_err(|e| {
            println!("jsonreader error: {:?}", e);
            return XCTestError::Polars(e)
        })?;

    println!("json report: {:?}", df);

    Ok(df)
}

fn load_squads_file(filepath: &PathBuf) -> Result<DataFrame, XCTestError> {
    let df = CsvReader::from_path(filepath)
        .map_err(|e| {
            println!("csvreader err #1: {:?}", e);
            return XCTestError::Polars(e)
        })?
        .has_header(true)
        .finish()
        .map_err(|e| {
            println!("csvreader err #2: {:?}", e);
            return XCTestError::Polars(e)
        })?;

    println!("csv headers: {:?}", df.fields());

    // TODO: Check if fields are correct
    let fields = df.fields()
        .iter()
        .map(|f| f.name.to_string())
        .collect::<Vec<String>>();

    let expected_field_names = ["ID", "Squad", "Filename"];
    for field in expected_field_names {
        if !fields.contains(&field.to_string()) {
            return Err(XCTestError::CSVParseError(CSVParseError::ColumnMissing { name: field }))
        }
    }

    Ok(df)
}

fn parse_file(arg: &str, extension: &str) -> Result<PathBuf, XCTestError> {
    let path = PathBuf::from(arg);
    let path_exists = path.try_exists().unwrap_or_default();

    if !path_exists {
        return Err(XCTestError::FilePathError(FilePathError::NotFound))
    }

    if path.extension() != Some(OsStr::new(extension)) {
        let extension = path.extension()
            .unwrap_or(OsStr::new("N/A"))
            .to_os_string()
            .into_string()
            .unwrap_or(String::from("N/A"));

        return Err(XCTestError::FilePathError(FilePathError::InvalidType { extension }))
    }

    Ok(path)
}

fn parse_xcresult_file(arg: &str) -> Result<PathBuf, XCTestError> {
    parse_file(arg, "xcresult")
}

fn parse_input_file(arg: &str) -> Result<PathBuf, XCTestError> {
    parse_file(arg, "csv")
}