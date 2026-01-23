use std::{collections::HashMap, time::SystemTime};

use just_enough_vcs::vcs::data::{
    local::{
        latest_file_data::LatestFileData,
        local_sheet::LocalSheetData,
        workspace_analyzer::{AnalyzeResultPure, ModifiedRelativePathBuf},
    },
    member::MemberId,
    sheet::SheetName,
};

use crate::outputs::status::JVStatusWrongModifyReason;

pub struct JVStatusInput {
    pub current_account: MemberId,
    pub current_sheet: SheetName,
    pub is_host_mode: bool,
    pub in_ref_sheet: bool,
    pub analyzed_result: AnalyzeResultPure,
    pub latest_file_data: LatestFileData,
    pub local_sheet_data: LocalSheetData,
    pub wrong_modified_items: HashMap<ModifiedRelativePathBuf, JVStatusWrongModifyReason>,
    pub update_time: SystemTime,
    pub now_time: SystemTime,
}

impl Default for JVStatusInput {
    fn default() -> Self {
        Self {
            current_account: MemberId::default(),
            current_sheet: SheetName::default(),
            is_host_mode: false,
            in_ref_sheet: false,
            analyzed_result: AnalyzeResultPure::default(),
            latest_file_data: LatestFileData::default(),
            local_sheet_data: LocalSheetData::default(),
            wrong_modified_items: HashMap::new(),
            update_time: SystemTime::now(),
            now_time: SystemTime::now(),
        }
    }
}
