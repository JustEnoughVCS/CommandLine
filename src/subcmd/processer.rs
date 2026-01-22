use crate::subcmd::cmd::JVCommandContext;
use crate::subcmd::cmds::_registry::{jv_cmd_nodes, jv_cmd_process_node};
use crate::subcmd::errors::CmdProcessError;
use crate::subcmd::renderer::JVRenderResult;

pub async fn jv_cmd_process(
    args: Vec<String>,
    ctx: JVCommandContext,
) -> Result<JVRenderResult, CmdProcessError> {
    let nodes = jv_cmd_nodes();
    let command = args.join(" ");

    // Find nodes that match the beginning of the command
    let matching_nodes: Vec<&String> = nodes
        .iter()
        .filter(|node| command.starts_with(node.as_str()))
        .collect();

    match matching_nodes.len() {
        0 => {
            // No matching node found
            return Err(CmdProcessError::NoMatchingCommand);
        }
        1 => {
            let matched_prefix = matching_nodes[0];
            let prefix_len = matched_prefix.split_whitespace().count();
            let trimmed_args: Vec<String> = args.into_iter().skip(prefix_len).collect();
            return jv_cmd_process_node(matched_prefix, trimmed_args, ctx).await;
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
