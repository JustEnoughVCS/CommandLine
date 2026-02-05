use just_enough_vcs::lib::data::{member::MemberId, vault::virtual_file::VirtualFileVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HereJsonResult {
    pub items: Vec<HereJsonResultItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HereJsonResultItem {
    pub mapping: String,
    pub name: String,
    pub current_version: VirtualFileVersion,
    pub size: usize,
    pub is_dir: bool,
    pub exist: bool,
    pub modified: bool,
    pub holder: MemberId,
}
