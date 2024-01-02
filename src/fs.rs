use std::fs;
use std::path::PathBuf;
use crate::err::{DirPathError, XCTestError};

pub fn derived_data_path() -> Result<PathBuf, XCTestError> {
    let xctest_home = home_path()?;
    Ok(PathBuf::from_iter([&xctest_home, &PathBuf::from("derived_data")]))
}

pub fn home_path() -> Result<PathBuf, XCTestError> {
    let home_path = home::home_dir().ok_or(XCTestError::DirPath(DirPathError::NotFound))?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from(".xctest")]))
}

pub fn xcresult_path(identifier: &String) -> Result<PathBuf, XCTestError> {
    let home_path = home_path()?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from(identifier), &PathBuf::from("result.xcresult")]))
}

pub fn full_report_path(identifier: &String) -> Result<PathBuf, XCTestError> {
    let home_path = home_path()?;
    Ok(
        PathBuf::from_iter([
            &home_path,
            &PathBuf::from(&identifier),
            &PathBuf::from("full_report.csv")
        ])
    )
}

pub fn report_path(identifier: &String) -> Result<PathBuf, XCTestError> {
    let home_path = home_path()?;
    Ok(
        PathBuf::from_iter([
            &home_path,
            &PathBuf::from(&identifier),
            &PathBuf::from("report.csv")
        ])
    )
}

pub fn get_workdir(identifier: &String) -> Result<PathBuf, XCTestError> {
    let home_path = home_path()?;
    let path = PathBuf::from_iter([
        &home_path,
        &PathBuf::from(&identifier)
    ]);

    fs::create_dir_all(&path)
        .map_err(|e| XCTestError::FileIO(e))?;

    Ok(path)
}

pub fn get_identifier() -> Result<String, XCTestError> {
    let identifier = chrono::offset::Local::now()
        .format("%F-%H-%M-%S")
        .to_string();

    get_workdir(&identifier)?;

    return Ok(identifier)
}