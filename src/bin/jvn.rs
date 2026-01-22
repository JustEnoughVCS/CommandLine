use just_enough_vcs_cli::cmd::cmd_system::JVCommandContext;
use just_enough_vcs_cli::utils::display::md;
use just_enough_vcs_cli::{
    cmd::{errors::CmdProcessError, processer::jv_cmd_process},
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

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let no_error_logs = special_flag!(args, "--no-error-logs");
    let quiet = special_flag!(args, "--quiet") || special_flag!(args, "-q");
    let help = special_flag!(args, "--help") || special_flag!(args, "-h");
    let confirmed = special_flag!(args, "--confirm") || special_flag!(args, "-C");

    // Process commands
    let render_result = match jv_cmd_process(args, JVCommandContext { help, confirmed }).await {
        Ok(result) => result,
        Err(e) => {
            if !no_error_logs {
                match e {
                    CmdProcessError::Prepare(cmd_prepare_error) => {
                        eprintln!(
                            "{}",
                            md(t!("process_error.prepare_error", error = cmd_prepare_error))
                        );
                    }
                    CmdProcessError::Execute(cmd_execute_error) => {
                        eprintln!(
                            "{}",
                            md(t!("process_error.execute_error", error = cmd_execute_error))
                        );
                    }
                    CmdProcessError::Render(cmd_render_error) => {
                        eprintln!(
                            "{}",
                            md(t!("process_error.render_error", error = cmd_render_error))
                        );
                    }
                    CmdProcessError::Error(error) => {
                        eprintln!("{}", md(t!("process_error.other", error = error)));
                    }
                    CmdProcessError::NoNodeFound(node) => {
                        eprintln!("{}", md(t!("process_error.no_node_found", node = node)));
                    }
                    CmdProcessError::NoMatchingCommand => {
                        eprintln!("{}", md(t!("process_error.no_matching_command")));
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
