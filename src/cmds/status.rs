use std::time::SystemTime;

use just_enough_vcs::vcs::constants::VAULT_HOST_NAME;

use crate::{
    arguments::status::JVStatusArgument,
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

impl JVCommand<JVStatusArgument, JVStatusInput, JVStatusOutput, JVStatusRenderer>
    for JVStatusCommand
{
    async fn prepare(
        _args: JVStatusArgument,
        _ctx: JVCommandContext,
    ) -> Result<JVStatusInput, CmdPrepareError> {
        // Initialize a reader for the local workspace and a default result structure
        let mut reader = LocalWorkspaceReader::default();
        let mut input = JVStatusInput::default();

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
        input.current_account = account;
        input.current_sheet = sheet_name;
        input.is_host_mode = is_host_mode;
        input.in_ref_sheet = is_ref_sheet;
        input.analyzed_result = analyzed;
        input.update_time = update_time;
        input.now_time = now_time;
        input.latest_file_data = latest_file_data;

        Ok(input)
    }

    async fn exec(mut input: JVStatusInput) -> Result<JVStatusOutput, CmdExecuteError> {
        let latest_file_data = &input.latest_file_data;

        // Calculate whether modifications are correc
        let modified = &input.analyzed_result.modified;
        for item in modified {
            // Get mapping
            let Ok(mapping) = input.local_sheet_data.mapping_data(&item) else {
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
                    input.wrong_modified_items.insert(
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
            if input.current_account != VAULT_HOST_NAME {
                let holder = latest_file_data.file_holder(mapping.mapping_vfid());
                if holder.is_none() {
                    input
                        .wrong_modified_items
                        .insert(item.clone(), JVStatusWrongModifyReason::NoHolder);
                    continue;
                }

                let holder = holder.cloned().unwrap();
                if &input.current_account != &holder {
                    input.wrong_modified_items.insert(
                        item.clone(),
                        JVStatusWrongModifyReason::ModifiedButNotHeld { holder: holder },
                    );
                }
            }
        }

        let output = JVStatusOutput {
            current_account: input.current_account,
            current_sheet: input.current_sheet,
            is_host_mode: input.is_host_mode,
            in_ref_sheet: input.in_ref_sheet,
            analyzed_result: input.analyzed_result,
            wrong_modified_items: input.wrong_modified_items,
            update_time: input.update_time,
            now_time: input.now_time,
        };

        Ok(output)
    }

    fn get_help_str() -> String {
        "".to_string()
    }
}
