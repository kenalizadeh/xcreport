use std::path::PathBuf;
use std::ops::{Div, Mul};
use polars::frame::DataFrame;
use polars::prelude::*;

use crate::errors::XCTestError;
use crate::fs::{raw_report_path, report_path};

pub fn process_raw_report(report: DataFrame) -> Result<DataFrame, XCTestError> {
    report
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
        .map_err(|e| XCTestError::Polars(e))
}

pub fn process_report(report: &DataFrame) -> Result<DataFrame, XCTestError> {
    report.clone()
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
        .map_err(|e| XCTestError::Polars(e))
}

pub fn save_raw_report(df: &mut DataFrame, identifier: &String) -> Result<(), XCTestError> {
    let raw_report_path = raw_report_path(&identifier)?;

    save_dataframe_csv(df, raw_report_path)
}

pub fn save_report(df: &mut DataFrame, identifier: &String) -> Result<(), XCTestError> {
    let report_path = report_path(&identifier)?;

    save_dataframe_csv(df, report_path)
}

fn save_dataframe_csv(df: &mut DataFrame, path: PathBuf) -> Result<(), XCTestError> {
    let mut file = std::fs::File::create(&path)
        .map_err(|e| XCTestError::FileIO(e))?;

    CsvWriter::new(&mut file)
        .finish(df)
        .map_err(|e| XCTestError::Polars(e))
}