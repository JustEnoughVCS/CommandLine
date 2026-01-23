use std::{collections::HashMap, time::SystemTime};

use just_enough_vcs::vcs::{
    constants::VAULT_HOST_NAME, data::local::workspace_analyzer::ModifiedRelativePathBuf,
};

use crate::{
    arguments::status::JVStatusArgument,
    collects::status::JVStatusCollect,
    inputs::status::JVStatusInput,
    outputs::status::{JVStatusOutput, JVStatusWrongModifyReason},
    renderers::status::JVStatusRenderer,
    systems::cmd::{
        cmd_system::{JVCommand, JVCommandContext},
        errors::{CmdExecuteError, CmdPrepareError},
    },
    utils::workspace_reader::LocalWorkspaceReader,
};

pub struct JVStatusCommand;

impl JVCommand<JVStatusArgument, JVStatusInput, JVStatusCollect, JVStatusOutput, JVStatusRenderer>
    for JVStatusCommand
{
    async fn prepare(
        _args: &JVStatusArgument,
        _ctx: &JVCommandContext,
    ) -> Result<JVStatusInput, CmdPrepareError> {
        Ok(JVStatusInput)
    }

    async fn collect(
        _args: &JVStatusArgument,
        _ctx: &JVCommandContext,
    ) -> Result<JVStatusCollect, CmdPrepareError> {
        // Initialize a reader for the local workspace and a default result structure
        let mut reader = LocalWorkspaceReader::default();
        let mut collect = JVStatusCollect::default();

        // Analyze the current status of the local workspace
        // (detects changes like created, modified, moved, etc.)
        let analyzed = reader.analyze_local_status().await?;

        // Retrieve the current account (member) ID
        let account = reader.current_account().await?;

        // Retrieve the name of the current sheet
        let sheet_name = reader.sheet_name().await?;

        // Is Host Mode
        let is_host_mode = reader.is_host_mode().await?;

        let cached_sheet = reader.cached_sheet(&sheet_name).await?;
        let sheet_holder = cached_sheet.holder().cloned().unwrap_or_default();
        let is_ref_sheet = sheet_holder == VAULT_HOST_NAME;

        // Get Latest file data
        let latest_file_data = reader.pop_latest_file_data(&account).await?;

        // Get the timestamp of the last update, defaulting to the current time if not available
        let update_time = reader
            .latest_info()
            .await?
            .update_instant
            .unwrap_or(SystemTime::now());

        // Record the current system time
        let now_time = SystemTime::now();

        // Populate the result structure with the gathered data
        collect.current_account = account;
        collect.current_sheet = sheet_name;
        collect.is_host_mode = is_host_mode;
        collect.in_ref_sheet = is_ref_sheet;
        collect.analyzed_result = analyzed;
        collect.update_time = update_time;
        collect.now_time = now_time;
        collect.latest_file_data = latest_file_data;
        Ok(collect)
    }

    async fn exec(
        _input: JVStatusInput,
        collect: JVStatusCollect,
    ) -> Result<JVStatusOutput, CmdExecuteError> {
        let mut wrong_modified_items: HashMap<ModifiedRelativePathBuf, JVStatusWrongModifyReason> =
            HashMap::new();

        let latest_file_data = &collect.latest_file_data;

        // Calculate whether modifications are correc
        let modified = &collect.analyzed_result.modified;
        for item in modified {
            // Get mapping
            let Ok(mapping) = collect.local_sheet_data.mapping_data(&item) else {
                continue;
            };

            // Check base version
            {
                let base_version = mapping.version_when_updated().clone();
                let Some(latest_version) = latest_file_data
                    .file_version(mapping.mapping_vfid())
                    .cloned()
                else {
                    continue;
                };

                // Base version dismatch
                if base_version != latest_version {
                    wrong_modified_items.insert(
                        item.clone(),
                        JVStatusWrongModifyReason::BaseVersionMismatch {
                            base_version,
                            latest_version,
                        },
                    );
                    continue;
                }
            }

            // Check edit right (only check when current is not HOST)
            if collect.current_account != VAULT_HOST_NAME {
                let holder = latest_file_data.file_holder(mapping.mapping_vfid());
                if holder.is_none() {
                    wrong_modified_items.insert(item.clone(), JVStatusWrongModifyReason::NoHolder);
                    continue;
                }

                let holder = holder.cloned().unwrap();
                if &collect.current_account != &holder {
                    wrong_modified_items.insert(
                        item.clone(),
                        JVStatusWrongModifyReason::ModifiedButNotHeld { holder: holder },
                    );
                }
            }
        }

        let output = JVStatusOutput {
            current_account: collect.current_account,
            current_sheet: collect.current_sheet,
            is_host_mode: collect.is_host_mode,
            in_ref_sheet: collect.in_ref_sheet,
            analyzed_result: collect.analyzed_result,
            wrong_modified_items: wrong_modified_items,
            update_time: collect.update_time,
            now_time: collect.now_time,
        };

        Ok(output)
    }

    fn get_help_str() -> String {
        "".to_string()
    }
}
