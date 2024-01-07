use std::path::PathBuf;
use std::ops::{Div, Mul};
use polars::frame::DataFrame;
use polars::prelude::*;

use crate::err::XCReportError;
use crate::fs::{full_report_path, report_path};

pub fn process_full_report(report: DataFrame) -> Result<DataFrame, XCReportError> {
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
                .fill_null(Expr::Literal(LiteralValue::String(String::from("N/A"))))
        )
        .collect()
        .map_err(XCReportError::Polars)
}

pub fn process_report(report: &DataFrame) -> Result<DataFrame, XCReportError> {
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
                .fill_null(Expr::Literal(LiteralValue::String(String::from("N/A"))))
        )
        .rename(["count"], ["Count"])
        .collect()
        .map_err(XCReportError::Polars)
}

pub fn save_full_report(df: &mut DataFrame, identifier: &String) -> Result<PathBuf, XCReportError> {
    let full_report_path = full_report_path(identifier)?;

    save_dataframe_csv(df, &full_report_path)?;

    Ok(full_report_path)
}

pub fn save_report_to_default(df: &mut DataFrame, identifier: &String) -> Result<PathBuf, XCReportError> {
    let report_path = report_path(identifier)?;

    save_dataframe_csv(df, &report_path)?;

    Ok(report_path)
}

pub fn save_report_to_output(df: &mut DataFrame, output_path: &PathBuf) -> Result<(), XCReportError> {
    save_dataframe_csv(df, output_path)
}

fn save_dataframe_csv(df: &mut DataFrame, path: &PathBuf) -> Result<(), XCReportError> {
    let mut file = std::fs::File::create(path)
        .map_err(XCReportError::FileIO)?;

    CsvWriter::new(&mut file)
        .finish(df)
        .map_err(XCReportError::Polars)
}