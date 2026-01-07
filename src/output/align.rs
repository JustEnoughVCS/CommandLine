use std::{collections::HashMap, path::PathBuf};

use just_enough_vcs::vcs::data::local::align::AlignTaskName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AlignJsonResult {
    pub align_tasks: HashMap<AlignTaskName, AlignTaskMapping>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AlignTaskMapping {
    pub local_mapping: PathBuf,
    pub remote_mapping: PathBuf,
}
