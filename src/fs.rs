use std::fs;
use std::path::PathBuf;
use crate::err::{DirPathError, XCReportError};

pub fn derived_data_path() -> Result<PathBuf, XCReportError> {
    let home_path = home_path()?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from("derived_data")]))
}

pub fn home_path() -> Result<PathBuf, XCReportError> {
    let home_path = home::home_dir().ok_or(XCReportError::DirPath(DirPathError::NotFound))?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from(".xcreport")]))
}

pub fn xcresult_path(identifier: &String) -> Result<PathBuf, XCReportError> {
    let home_path = home_path()?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from(identifier), &PathBuf::from("result.xcresult")]))
}

pub fn xcpretty_report_path(identifier: &String) -> Result<PathBuf, XCReportError> {
    let home_path = home_path()?;

    Ok(PathBuf::from_iter([&home_path, &PathBuf::from(identifier), &PathBuf::from("xcpretty_report.html")]))
}

pub fn full_report_path(identifier: &String) -> Result<PathBuf, XCReportError> {
    let home_path = home_path()?;
    Ok(
        PathBuf::from_iter([
            &home_path,
            &PathBuf::from(&identifier),
            &PathBuf::from("full_report.csv")
        ])
    )
}

pub fn report_path(identifier: &String) -> Result<PathBuf, XCReportError> {
    let home_path = home_path()?;
    Ok(
        PathBuf::from_iter([
            &home_path,
            &PathBuf::from(&identifier),
            &PathBuf::from("report.csv")
        ])
    )
}

pub fn get_workdir(identifier: &String) -> Result<PathBuf, XCReportError> {
    let home_path = home_path()?;
    let path = PathBuf::from_iter([
        &home_path,
        &PathBuf::from(&identifier)
    ]);

    fs::create_dir_all(&path)
        .map_err(XCReportError::FileIO)?;

    Ok(path)
}

pub fn get_identifier() -> Result<String, XCReportError> {
    let identifier = chrono::offset::Local::now()
        .format("%F-%H-%M-%S")
        .to_string();

    get_workdir(&identifier)?;

    Ok(identifier)
}