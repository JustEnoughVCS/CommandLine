use std::collections::HashMap;

use just_enough_vcs::vcs::data::{
    member::MemberId,
    sheet::{SheetMappingMetadata, SheetPathBuf},
    vault::mapping_share::SheetShareId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShareListResult {
    pub share_list: Vec<ShareItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShareItem {
    pub share_id: SheetShareId,
    pub sharer: MemberId,
    pub description: String,
    pub file_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SeeShareResult {
    pub share_id: SheetShareId,
    pub sharer: MemberId,
    pub description: String,
    pub mappings: HashMap<SheetPathBuf, SheetMappingMetadata>,
}
