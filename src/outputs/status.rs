use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::SystemTime,
};

use just_enough_vcs::vcs::data::{
    local::workspace_analyzer::{
        CreatedRelativePathBuf, FromRelativePathBuf, LostRelativePathBuf, ModifiedRelativePathBuf,
        ToRelativePathBuf,
    },
    member::MemberId,
    sheet::SheetName,
    vault::virtual_file::VirtualFileId,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct JVStatusResult {
    pub current_account: MemberId,
    pub current_sheet: SheetName,
    pub moved: HashMap<VirtualFileId, (FromRelativePathBuf, ToRelativePathBuf)>,
    pub created: HashSet<CreatedRelativePathBuf>,
    pub lost: HashSet<LostRelativePathBuf>,
    pub erased: HashSet<PathBuf>,
    pub modified: HashSet<ModifiedRelativePathBuf>,
    pub update_time: SystemTime,
    pub now_time: SystemTime,
}

impl Default for JVStatusResult {
    fn default() -> Self {
        Self {
            current_account: MemberId::default(),
            current_sheet: SheetName::default(),
            moved: HashMap::default(),
            created: HashSet::default(),
            lost: HashSet::default(),
            erased: HashSet::default(),
            modified: HashSet::default(),
            update_time: SystemTime::now(),
            now_time: SystemTime::now(),
        }
    }
}
