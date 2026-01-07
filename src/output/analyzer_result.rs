use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AnalyzerJsonResult {
    pub created: Vec<PathBuf>,
    pub lost: Vec<PathBuf>,
    pub erased: Vec<PathBuf>,
    pub moved: Vec<(PathBuf, PathBuf)>,
    pub modified: Vec<(PathBuf, ModifiedType)>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum ModifiedType {
    Modified,
    ModifiedButBaseVersionMismatch,
    ModifiedButNotHeld,
}
