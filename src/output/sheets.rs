use just_enough_vcs::vcs::data::member::MemberId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SheetListJsonResult {
    pub my_sheets: Vec<SheetItem>,
    pub reference_sheets: Vec<SheetItem>,
    pub other_sheets: Vec<SheetItem>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SheetItem {
    pub name: String,
    pub holder: MemberId,
}
