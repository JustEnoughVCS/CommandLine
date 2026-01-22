use std::time::SystemTime;

use crate::{
    arguments::status::JVStatusArgument,
    outputs::status::JVStatusResult,
    renderers::status::JVStatusRenderer,
    systems::cmd::{
        cmd_system::{JVCommand, JVCommandContext},
        errors::{CmdExecuteError, CmdPrepareError},
    },
    utils::workspace_reader::LocalWorkspaceReader,
};

pub struct JVStatusCommand;

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
