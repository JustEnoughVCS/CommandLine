use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::SystemTime,
};

use clap::Parser;
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

use crate::{
    cmd::{
        cmd_system::{JVCommand, JVCommandContext},
        errors::{CmdExecuteError, CmdPrepareError, CmdRenderError},
        renderer::{JVRenderResult, JVResultRenderer},
    },
    utils::workspace_reader::LocalWorkspaceReader,
};

pub struct JVStatusCommand;

#[derive(Parser, Debug)]
pub struct JVStatusArgument;

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

impl JVCommand<JVStatusArgument, JVStatusResult, JVStatusResult, JVStatusRenderer>
    for JVStatusCommand
{
    async fn prepare(
        _args: JVStatusArgument,
        _ctx: JVCommandContext,
    ) -> Result<JVStatusResult, CmdPrepareError> {
        // Initialize a reader for the local workspace and a default result structure
        let mut reader = LocalWorkspaceReader::default();
        let mut input = JVStatusResult::default();

        // Analyze the current status of the local workspace
        // (detects changes like created, modified, moved, etc.)
        let analyzed = reader.analyze_local_status().await?;

        // Retrieve the current account (member) ID
        let account = reader.current_account().await?;

        // Retrieve the name of the current sheet
        let sheet_name = reader.sheet_name().await?;

        // Get the timestamp of the last update, defaulting to the current time if not available
        let update_time = reader
            .latest_info()
            .await?
            .update_instant
            .unwrap_or(SystemTime::now());

        // Record the current system time
        let now_time = SystemTime::now();

        // Populate the result structure with the gathered data
        input.current_account = account;
        input.current_sheet = sheet_name;
        input.moved = analyzed.moved;
        input.created = analyzed.created;
        input.lost = analyzed.lost;
        input.erased = analyzed.erased;
        input.modified = analyzed.modified;
        input.update_time = update_time;
        input.now_time = now_time;

        Ok(input)
    }

    async fn exec(input: JVStatusResult) -> Result<JVStatusResult, CmdExecuteError> {
        Ok(input) // Analyze command, no needs execute
    }

    fn get_help_str() -> String {
        "".to_string()
    }
}

pub struct JVStatusRenderer;

impl JVResultRenderer<JVStatusResult> for JVStatusRenderer {
    async fn render(data: &JVStatusResult) -> Result<JVRenderResult, CmdRenderError> {
        todo!()
    }
}
