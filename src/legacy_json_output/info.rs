use just_enough_vcs::vcs::data::{
    member::MemberId,
    vault::virtual_file::{VirtualFileId, VirtualFileVersion},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InfoJsonResult {
    pub mapping: String,
    pub in_ref: String,
    pub vfid: VirtualFileId,
    pub histories: Vec<InfoHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InfoHistory {
    pub version: VirtualFileVersion,
    pub version_creator: MemberId,
    pub version_description: String,
    pub is_current_version: bool,
    pub is_ref_version: bool,
}
