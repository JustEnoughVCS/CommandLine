use std::{collections::HashMap, time::SystemTime};

use just_enough_vcs::vcs::data::{
    local::workspace_analyzer::{AnalyzeResultPure, ModifiedRelativePathBuf},
    member::MemberId,
    sheet::SheetName,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct JVStatusOutput {
    pub current_account: MemberId,
    pub current_sheet: SheetName,
    pub is_host_mode: bool,
    pub in_ref_sheet: bool,
    pub analyzed_result: AnalyzeResultPure,
    pub wrong_modified_items: HashMap<ModifiedRelativePathBuf, JVStatusWrongModifyReason>,
    pub update_time: SystemTime,
    pub now_time: SystemTime,
}

#[derive(Serialize)]
pub enum JVStatusWrongModifyReason {
    BaseVersionMismatch {
        base_version: String,
        latest_version: String,
    },
    ModifiedButNotHeld {
        holder: String,
    },
    NoHolder,
}

impl Default for JVStatusOutput {
    fn default() -> Self {
        Self {
            current_account: MemberId::default(),
            current_sheet: SheetName::default(),
            is_host_mode: false,
            in_ref_sheet: false,
            analyzed_result: AnalyzeResultPure::default(),
            wrong_modified_items: HashMap::new(),
            update_time: SystemTime::now(),
            now_time: SystemTime::now(),
        }
    }
}
