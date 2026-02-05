use std::time::SystemTime;

use just_enough_vcs::lib::data::{
    local::{
        latest_file_data::LatestFileData, local_sheet::LocalSheetData,
        workspace_analyzer::AnalyzeResultPure,
    },
    member::MemberId,
    sheet::SheetName,
};

pub struct JVStatusCollect {
    pub current_account: MemberId,
    pub current_sheet: SheetName,
    pub is_host_mode: bool,
    pub in_ref_sheet: bool,
    pub analyzed_result: AnalyzeResultPure,
    pub latest_file_data: LatestFileData,
    pub local_sheet_data: LocalSheetData,
    pub update_time: SystemTime,
    pub now_time: SystemTime,
}

impl Default for JVStatusCollect {
    fn default() -> Self {
        Self {
            current_account: MemberId::default(),
            current_sheet: SheetName::default(),
            is_host_mode: false,
            in_ref_sheet: false,
            analyzed_result: AnalyzeResultPure::default(),
            latest_file_data: LatestFileData::default(),
            local_sheet_data: LocalSheetData::default(),
            update_time: SystemTime::now(),
            now_time: SystemTime::now(),
        }
    }
}
