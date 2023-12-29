use clap::builder::Str;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct XCodeBuildReport {
    targets: Vec<Target>
}

impl XCodeBuildReport {
    pub fn get_all_files(&self) -> Vec<&TargetFile> {
        self.targets
            .iter()
            .flat_map(|t| &t.files)
            .collect()
    }
}

#[derive(Deserialize, Debug)]
pub struct Target {
    #[serde(rename(deserialize = "coveredLines"))]
    covered_lines: usize,
    #[serde(rename(deserialize = "executableLines"))]
    executable_lines: usize,
    #[serde(rename(deserialize = "lineCoverage"))]
    line_coverage: f32,
    files: Vec<TargetFile>
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TargetFile {
    path: String,
    #[serde(rename(deserialize = "coveredLines"))]
    covered_lines: usize,
    #[serde(rename(deserialize = "executableLines"))]
    executable_lines: usize,
    #[serde(rename(deserialize = "lineCoverage"))]
    line_coverage: f32,
    squad_name: Option<String>
}

impl TargetFile {

    pub fn file_path(&self) -> &String {
        &self.path
    }
    pub fn set_squad_name(&mut self, name: String) {
        self.squad_name = Some(name)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SquadData {
    #[serde(rename(deserialize = "Squad"))]
    squad_name: String,
    #[serde(rename(deserialize = "Filepath"))]
    file_path: String
}

impl SquadData {
    pub fn file_name(&self) -> &String {
        &self.file_path
    }

    pub fn squad_name(&self) -> &String {
        &self.squad_name
    }
}