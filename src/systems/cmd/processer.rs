use crate::systems::cmd::_commands::{jv_cmd_nodes, jv_cmd_process_node};
use crate::systems::cmd::cmd_system::JVCommandContext;
use crate::systems::cmd::errors::CmdProcessError;
use crate::systems::render::renderer::JVRenderResult;

pub async fn jv_cmd_process(
    args: &Vec<String>,
    ctx: JVCommandContext,
    renderer_override: String,
) -> Result<JVRenderResult, CmdProcessError> {
    let nodes = jv_cmd_nodes();

    // Why add a space?
    // Add a space at the end of the command string for precise command prefix matching.
    // For example: when the input command is "bananas", if there are two commands "banana" and "bananas",
    // without a space it might incorrectly match "banana" (because "bananas".starts_with("banana") is true).
    // After adding a space, "bananas " will not match "banana ", thus avoiding ambiguity caused by overlapping prefixes.
    let command = format!("{} ", args.join(" "));

    // Find all nodes that match the command prefix
    let matching_nodes: Vec<&String> = nodes
        .iter()
        // Also add a space to the node string to ensure consistent matching logic
        .filter(|node| command.starts_with(&format!("{} ", node)))
        .collect();

    match matching_nodes.len() {
        0 => {
            // No matching node found
            return Err(CmdProcessError::NoMatchingCommand);
        }
        1 => {
            let matched_prefix = matching_nodes[0];
            let prefix_len = matched_prefix.split_whitespace().count();
            let trimmed_args: Vec<String> = args.into_iter().cloned().skip(prefix_len).collect();
            return jv_cmd_process_node(matched_prefix, trimmed_args, ctx, renderer_override).await;
        }
        _ => {
            // Multiple matching nodes found
            return Err(CmdProcessError::AmbiguousCommand(
                matching_nodes
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            ));
        }
    }
}
