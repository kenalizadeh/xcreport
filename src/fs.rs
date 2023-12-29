use std::fs;
use std::path::PathBuf;
use crate::errors::XCTestError;
use crate::errors::DirPathError;

pub fn home_path() -> Result<PathBuf, XCTestError> {
    let home_path = home::home_dir().ok_or(XCTestError::DirPath(DirPathError::NotFound))?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from(".xctest")]))
}

pub fn xcresult_path() -> Result<PathBuf, XCTestError> {
    let home_path = home_path()?;
    Ok(PathBuf::from_iter([&home_path, &PathBuf::from("result.xcresult")]))
}

pub fn setup_home_dir() -> Result<(), XCTestError> {
    let xctest_home = home_path()?;

    fs::create_dir_all(&xctest_home)
        .map_err(|e| {
            println!("setup_home_dir create error: {:?}", e);
            return XCTestError::FileIO(e)
        })?;

    Ok(())
}