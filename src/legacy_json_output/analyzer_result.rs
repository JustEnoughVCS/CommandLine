use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AnalyzerJsonResult {
    pub created: Vec<PathBuf>,
    pub lost: Vec<PathBuf>,
    pub erased: Vec<PathBuf>,
    pub moved: Vec<MovedItem>,
    pub modified: Vec<ModifiedItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum ModifiedType {
    Modified,
    ModifiedButBaseVersionMismatch,
    ModifiedButNotHeld,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MovedItem {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModifiedItem {
    pub path: PathBuf,
    pub modification_type: ModifiedType,
}
