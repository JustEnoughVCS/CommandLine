use std::process::exit;

use just_enough_vcs_cli::systems::cmd::_registry::jv_cmd_nodes;
use just_enough_vcs_cli::systems::cmd::cmd_system::JVCommandContext;
use just_enough_vcs_cli::systems::cmd::errors::{CmdExecuteError, CmdPrepareError, CmdRenderError};
use just_enough_vcs_cli::utils::display::md;
use just_enough_vcs_cli::utils::levenshtein_distance::levenshtein_distance;
use just_enough_vcs_cli::{
    systems::cmd::{errors::CmdProcessError, processer::jv_cmd_process},
    utils::env::current_locales,
};
use rust_i18n::{set_locale, t};

rust_i18n::i18n!("resources/locales/jvn", fallback = "en");

macro_rules! special_flag {
    ($args:expr, $flag:expr) => {{
        let flag = $flag;
        let found = $args.iter().any(|arg| arg == flag);
        $args.retain(|arg| arg != flag);
        found
    }};
}

macro_rules! special_argument {
    ($args:expr, $flag:expr) => {{
        let flag = $flag;
        let mut value: Option<String> = None;
        let mut i = 0;
        while i < $args.len() {
            if $args[i] == flag {
                if i + 1 < $args.len() {
                    value = Some($args[i + 1].clone());
                    $args.remove(i + 1);
                    $args.remove(i);
                } else {
                    value = None;
                    $args.remove(i);
                }
                break;
            }
            i += 1;
        }
        value
    }};
}

#[tokio::main]
async fn main() {
    // Collect arguments
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    // Init i18n
    let lang = special_argument!(args, "--lang").unwrap_or(current_locales());
    set_locale(&lang);

    // Renderer
    let renderer_override = special_argument!(args, "--renderer").unwrap_or("default".to_string());

    // Other flags
    let no_error_logs = special_flag!(args, "--no-error-logs");
    let quiet = special_flag!(args, "--quiet") || special_flag!(args, "-q");
    let help = special_flag!(args, "--help") || special_flag!(args, "-h");
    let confirmed = special_flag!(args, "--confirm") || special_flag!(args, "-C");

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    // Handle help when no arguments provided
    if args.len() < 1 && help {
        eprintln!("{}", md(t!("help")));
        exit(1);
    }

    // Process commands
    let render_result = match jv_cmd_process(
        &args,
        JVCommandContext { help, confirmed },
        renderer_override,
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            if !no_error_logs {
                match e {
                    CmdProcessError::Prepare(cmd_prepare_error) => {
                        handle_prepare_error(cmd_prepare_error);
                    }
                    CmdProcessError::Execute(cmd_execute_error) => {
                        handle_execute_error(cmd_execute_error);
                    }
                    CmdProcessError::Render(cmd_render_error) => {
                        handle_render_error(cmd_render_error);
                    }
                    CmdProcessError::Error(error) => {
                        eprintln!("{}", md(t!("process_error.other", error = error)));
                    }
                    CmdProcessError::NoNodeFound(node) => {
                        eprintln!("{}", md(t!("process_error.no_node_found", node = node)));
                    }
                    CmdProcessError::NoMatchingCommand => {
                        handle_no_matching_command_error(args);
                    }
                    CmdProcessError::AmbiguousCommand(nodes) => {
                        let nodes_list = nodes
                            .iter()
                            .enumerate()
                            .map(|(i, node)| format!("{}. {}", i + 1, node))
                            .collect::<Vec<String>>()
                            .join("\n");
                        eprintln!(
                            "{}",
                            md(t!("process_error.ambiguous_command", nodes = nodes_list))
                        );
                    }
                    CmdProcessError::ParseError(help) => {
                        if help.trim().len() < 1 {
                            eprintln!("{}", md(t!("process_error.parse_error")));
                        } else {
                            eprintln!("{}", help)
                        }
                    }
                }
            }
            std::process::exit(1);
        }
    };

    // Print
    if !quiet {
        print!("{}", render_result);
    }
}

fn handle_no_matching_command_error(args: Vec<String>) {
    let mut similar_nodes: Vec<String> = Vec::new();
    for node in jv_cmd_nodes() {
        let node_len = node.split(" ").collect::<Vec<&str>>().iter().len();
        let args_len = args.len();
        if args_len < node_len {
            continue;
        }
        let args_str = args[..node_len].join(" ");
        let distance = levenshtein_distance(args_str.as_str(), node.as_str());
        if distance <= 2 {
            similar_nodes.push(node);
        }
    }
    if similar_nodes.len() < 1 {
        eprintln!("{}", md(t!("process_error.no_matching_command")));
    } else {
        eprintln!(
            "{}",
            md(t!(
                "process_error.no_matching_command_but_similar",
                similars = similar_nodes[0]
            ))
        );
    }
}

fn handle_prepare_error(cmd_prepare_error: CmdPrepareError) {
    match cmd_prepare_error {
        CmdPrepareError::Io(error) => {
            eprintln!("{}", md(t!("prepare_error.io", error = error.to_string())));
        }
        CmdPrepareError::Error(msg) => {
            eprintln!("{}", md(t!("prepare_error.error", error = msg)));
        }
        CmdPrepareError::LocalWorkspaceNotFound => {
            eprintln!("{}", md(t!("prepare_error.local_workspace_not_found")));
        }
        CmdPrepareError::LocalConfigNotFound => {
            eprintln!("{}", md(t!("prepare_error.local_config_not_found")));
        }
        CmdPrepareError::LatestInfoNotFound => {
            eprintln!("{}", md(t!("prepare_error.latest_info_not_found")));
        }
        CmdPrepareError::LatestFileDataNotExist(member_id) => {
            eprintln!(
                "{}",
                md(t!(
                    "prepare_error.latest_file_data_not_exist",
                    member_id = member_id
                ))
            );
        }
        CmdPrepareError::CachedSheetNotFound(sheet_name) => {
            eprintln!(
                "{}",
                md(t!(
                    "prepare_error.cached_sheet_not_found",
                    sheet_name = sheet_name
                ))
            );
        }
        CmdPrepareError::LocalSheetNotFound(member_id, sheet_name) => {
            eprintln!(
                "{}",
                md(t!(
                    "prepare_error.local_sheet_not_found",
                    member_id = member_id,
                    sheet_name = sheet_name
                ))
            );
        }
        CmdPrepareError::LocalStatusAnalyzeFailed => {
            eprintln!("{}", md(t!("prepare_error.local_status_analyze_failed")));
        }
        CmdPrepareError::NoSheetInUse => {
            eprintln!("{}", md(t!("prepare_error.no_sheet_in_use")));
        }
    }
}

fn handle_execute_error(cmd_execute_error: CmdExecuteError) {
    match cmd_execute_error {
        CmdExecuteError::Io(error) => {
            eprintln!("{}", md(t!("execute_error.io", error = error.to_string())));
        }
        CmdExecuteError::Prepare(cmd_prepare_error) => handle_prepare_error(cmd_prepare_error),
        CmdExecuteError::Error(msg) => {
            eprintln!("{}", md(t!("execute_error.error", error = msg)));
        }
    }
}

fn handle_render_error(cmd_render_error: CmdRenderError) {
    match cmd_render_error {
        CmdRenderError::Io(error) => {
            eprintln!("{}", md(t!("render_error.io", error = error.to_string())));
        }
        CmdRenderError::Prepare(cmd_prepare_error) => handle_prepare_error(cmd_prepare_error),
        CmdRenderError::Execute(cmd_execute_error) => handle_execute_error(cmd_execute_error),
        CmdRenderError::Error(msg) => {
            eprintln!("{}", md(t!("render_error.error", error = msg)));
        }
        CmdRenderError::SerializeFailed(error) => {
            eprintln!(
                "{}",
                md(t!(
                    "render_error.serialize_failed",
                    error = error.to_string()
                ))
            );
        }
        CmdRenderError::RendererNotFound(renderer_name) => {
            eprintln!(
                "{}",
                md(t!(
                    "render_error.renderer_not_found",
                    renderer_name = renderer_name
                ))
            );
        }
    }
}
