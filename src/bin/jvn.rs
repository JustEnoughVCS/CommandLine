use std::{
    ops::Deref,
    path::{Path, PathBuf},
    process::exit,
};

use cli_utils::{
    display::markdown::Markdown, env::locales::current_locales, math::levenshtein_distance,
};
use just_progress::{
    progress,
    renderer::{ProgressSimpleRenderer, RendererTheme},
};
use jvcli::{
    special_argument, special_flag,
    systems::{
        cmd::{
            _commands::jv_cmd_nodes,
            cmd_system::JVCommandContext,
            errors::{CmdExecuteError, CmdPrepareError, CmdProcessError, CmdRenderError},
            processer::jv_cmd_process,
        },
        debug::verbose_logger::init_verbose_logger,
        helpdoc::{
            DEFAULT_HELPDOC,
            helpdoc_viewer::{self},
        },
    },
};
use log::{LevelFilter, error, info, trace, warn};
use rust_i18n::{set_locale, t};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

rust_i18n::i18n!("resources/locales/jvn", fallback = "en");

#[tokio::main]
async fn main() {
    // Collect arguments
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    // Output control flags
    let quiet = special_flag!(args, "--quiet") || special_flag!(args, "-q");
    let verbose = special_flag!(args, "--verbose") || special_flag!(args, "-V");
    let no_progress = special_flag!(args, "--no-progress");
    let verbose_full = special_flag!(args, "--verbose-full");

    // If `--verbose` or `--verbose-full` is enabled and `--quiet` is not enabled, turn on the logger
    let filter = if (verbose || verbose_full) && !quiet {
        let filter = if verbose_full {
            LevelFilter::Trace
        } else {
            LevelFilter::Info
        };
        Some(filter)
    } else {
        None
    };
    init_verbose_logger(filter);
    trace!("{}", t!("verbose.setup_verbose"));

    // If `--no-progress` or `--quiet` is enabled, the progress bar will not be initialized
    let progress_future = if no_progress || quiet {
        None
    } else {
        trace!("{}", t!("verbose.setup_progress"));

        // Build progress future
        let progress_center = progress::init();
        let renderer = ProgressSimpleRenderer::new()
            .with_subprogress(true)
            .with_theme(RendererTheme {
                layout: "{name},{progress},{percent},{state}",
                ..Default::default()
            });
        Some(progress::bind(progress_center, move |name, state| {
            renderer.update(name, state)
        }))
    };

    // I18n flags
    let lang = special_argument!(args, "--lang").unwrap_or(current_locales());
    set_locale(&lang);
    trace!("{}", t!("verbose.setup_i18n", lang = lang));

    // Renderer
    let renderer_override = special_argument!(args, "--renderer").unwrap_or("default".to_string());
    trace!(
        "{}",
        t!("verbose.setup_renderer", renderer = renderer_override)
    );

    // Other flags
    let no_error_logs = special_flag!(args, "--no-error-logs");
    let help = special_flag!(args, "--help") || special_flag!(args, "-h");
    let version = special_flag!(args, "--version") || special_flag!(args, "-v");
    let confirmed = special_flag!(args, "--confirm") || special_flag!(args, "-C");

    if version {
        args.insert(0, "version".to_string());
        trace!("{}", t!("verbose.redirect_to_version_command"));
    }
    if no_error_logs {
        trace!("{}", t!("verbose.no_error_logs"));
    }
    if help {
        trace!("{}", t!("verbose.help"));
    }
    if confirmed {
        trace!("{}", t!("verbose.confirmed"));
    }

    // Handle help when no arguments provided
    if args.is_empty() {
        if help {
            warn!("{}", t!("verbose.no_arguments"));
            helpdoc_viewer::display_with_lang(DEFAULT_HELPDOC, &lang).await;
            exit(1);
        } else {
            eprintln!("{}", t!("process_error.empty_input").to_string().markdown());
            exit(1);
        }
    }

    info!("{}", t!("verbose.user_input", command = args.join(" ")));

    // Read pipe inpuit
    let (stdin_path, stdin_data) = match read_all_from_stdin().await {
        Ok((path, data)) => {
            if data.is_empty() {
                (None, None)
            } else {
                (path, Some(data))
            }
        }
        Err(_) => (None, None),
    };

    // Trace stdin result
    if let Some(ref path) = stdin_path {
        trace!("{}", t!("verbose.stdin_path", path = path.display()).trim());
    }
    if let Some(ref data) = stdin_data {
        trace!("{}", t!("verbose.stdin_data", len = data.len()).trim());
    }

    // Build process future
    let args_clone = args.clone();
    let process_future = jv_cmd_process(
        &args_clone,
        JVCommandContext {
            help,
            confirmed,
            args: args.clone(),
            lang,
            stdin_path,
            stdin_data,
        },
        renderer_override,
    );

    // Process commands
    let (_, process_result) = tokio::join!(
        async {
            if let Some(progress_future) = progress_future {
                progress_future.await;
            }
        },
        process_future
    );

    // Match result
    let render_result = match process_result {
        Ok(result) => {
            info!("{}", t!("verbose.process_success"));
            result
        }
        Err(e) => {
            error!("{}", t!("verbose.process_fail"));
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
                        eprintln!(
                            "{}",
                            t!("process_error.other", error = error)
                                .to_string()
                                .markdown()
                        );
                    }
                    CmdProcessError::NoNodeFound(node) => {
                        eprintln!(
                            "{}",
                            t!("process_error.no_node_found", node = node)
                                .to_string()
                                .markdown()
                        );
                    }
                    CmdProcessError::NoMatchingCommand => {
                        handle_no_matching_command_error(args);
                    }
                    CmdProcessError::ParseError(help) => {
                        if help.trim().is_empty() {
                            eprintln!("{}", t!("process_error.parse_error").to_string().markdown());
                        } else {
                            eprintln!("{}", help)
                        }
                    }
                    CmdProcessError::RendererOverrideButRequestHelp => {
                        eprintln!(
                            "{}",
                            t!("process_error.renderer_override_but_request_help")
                                .to_string()
                                .markdown()
                        );
                    }
                    CmdProcessError::DowncastFailed => {
                        eprintln!(
                            "{}",
                            t!("process_error.downcast_failed").to_string().markdown()
                        );
                    }
                }
            }
            std::process::exit(1);
        }
    };

    // Print
    if !quiet {
        info!("{}", t!("verbose.print_render_result"));
        let r = render_result.deref();
        if !r.is_empty() {
            print!("{}", r);
            if let Err(e) = io::stdout().flush().await
                && !no_error_logs
            {
                display_io_error(e);
            }
        }
    }
}

/// Read path or raw information from standard input
async fn read_all_from_stdin() -> tokio::io::Result<(Option<PathBuf>, Vec<u8>)> {
    if atty::is(atty::Stream::Stdin) {
        return Ok((None, Vec::new()));
    }

    let mut stdin = tokio::io::stdin();
    let mut buffer = Vec::new();

    stdin.read_to_end(&mut buffer).await?;

    if buffer.is_empty() {
        return Ok((None, Vec::new()));
    }

    let path = if let Ok(input_str) = String::from_utf8(buffer.clone()) {
        let trimmed = input_str.trim();
        if !trimmed.is_empty() && Path::new(trimmed).exists() {
            Some(PathBuf::from(trimmed))
        } else {
            None
        }
    } else {
        None
    };

    Ok((path, buffer))
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
        let distance = levenshtein_distance::levenshtein_distance(args_str.as_str(), node.as_str());
        if distance <= 2 {
            similar_nodes.push(node);
        }
    }
    if similar_nodes.is_empty() {
        eprintln!(
            "{}",
            t!("process_error.no_matching_command")
                .to_string()
                .markdown()
        );
    } else {
        eprintln!(
            "{}",
            t!(
                "process_error.no_matching_command_but_similar",
                similars = similar_nodes[0]
            )
            .to_string()
            .markdown()
        );
    }
}

fn handle_prepare_error(cmd_prepare_error: CmdPrepareError) {
    match cmd_prepare_error {
        CmdPrepareError::Io(error) => {
            eprintln!(
                "{}",
                t!("prepare_error.io", error = display_io_error(error))
                    .to_string()
                    .markdown()
            );
        }
        CmdPrepareError::Error(msg) => {
            eprintln!(
                "{}",
                t!("prepare_error.error", error = msg)
                    .to_string()
                    .markdown()
            );
        }
        CmdPrepareError::EarlyOutput(_) => {
            // Early output is not an error
            // No additional handling needed,
            //   this result has already been captured in `crate::systems::cmd::cmd_system.rs`
            exit(0)
        }
    }
}

fn handle_execute_error(cmd_execute_error: CmdExecuteError) {
    match cmd_execute_error {
        CmdExecuteError::Io(error) => {
            eprintln!(
                "{}",
                t!("execute_error.io", error = display_io_error(error))
                    .to_string()
                    .markdown()
            );
        }
        CmdExecuteError::Prepare(cmd_prepare_error) => handle_prepare_error(cmd_prepare_error),
        CmdExecuteError::Error(msg) => {
            eprintln!(
                "{}",
                t!("execute_error.error", error = msg)
                    .to_string()
                    .markdown()
            );
        }
    }
}

fn handle_render_error(cmd_render_error: CmdRenderError) {
    match cmd_render_error {
        CmdRenderError::Io(error) => {
            eprintln!(
                "{}",
                t!("render_error.io", error = display_io_error(error))
                    .to_string()
                    .markdown()
            );
        }
        CmdRenderError::Prepare(cmd_prepare_error) => handle_prepare_error(cmd_prepare_error),
        CmdRenderError::Execute(cmd_execute_error) => handle_execute_error(cmd_execute_error),
        CmdRenderError::Error(msg) => {
            eprintln!(
                "{}",
                t!("render_error.error", error = msg).to_string().markdown()
            );
        }
        CmdRenderError::SerializeFailed(error) => {
            eprintln!(
                "{}",
                t!("render_error.serialize_failed", error = error.to_string())
                    .to_string()
                    .markdown()
            );
        }
        CmdRenderError::RendererNotFound(renderer_name) => {
            eprintln!(
                "{}",
                t!(
                    "render_error.renderer_not_found",
                    renderer_name = renderer_name
                )
                .to_string()
                .markdown()
            );
        }
        CmdRenderError::TypeMismatch {
            expected: _,
            actual: _,
        } => {
            eprintln!(
                "{}",
                t!("render_error.type_mismatch").to_string().markdown()
            );
        }
    }
}

fn display_io_error(error: std::io::Error) -> std::borrow::Cow<'static, str> {
    match error.kind() {
        std::io::ErrorKind::NotFound => t!("io_error.not_found", raw_error = error),
        std::io::ErrorKind::PermissionDenied => t!("io_error.permission_denied", raw_error = error),
        std::io::ErrorKind::ConnectionRefused => {
            t!("io_error.connection_refused", raw_error = error)
        }
        std::io::ErrorKind::ConnectionReset => t!("io_error.connection_reset", raw_error = error),
        std::io::ErrorKind::HostUnreachable => t!("io_error.host_unreachable", raw_error = error),
        std::io::ErrorKind::NetworkUnreachable => {
            t!("io_error.network_unreachable", raw_error = error)
        }
        std::io::ErrorKind::ConnectionAborted => {
            t!("io_error.connection_aborted", raw_error = error)
        }
        std::io::ErrorKind::NotConnected => t!("io_error.not_connected", raw_error = error),
        std::io::ErrorKind::AddrInUse => t!("io_error.addr_in_use", raw_error = error),
        std::io::ErrorKind::AddrNotAvailable => {
            t!("io_error.addr_not_available", raw_error = error)
        }
        std::io::ErrorKind::NetworkDown => t!("io_error.network_down", raw_error = error),
        std::io::ErrorKind::BrokenPipe => t!("io_error.broken_pipe", raw_error = error),
        std::io::ErrorKind::AlreadyExists => t!("io_error.already_exists", raw_error = error),
        std::io::ErrorKind::WouldBlock => t!("io_error.would_block", raw_error = error),
        std::io::ErrorKind::NotADirectory => t!("io_error.not_a_directory", raw_error = error),
        std::io::ErrorKind::IsADirectory => t!("io_error.is_a_directory", raw_error = error),
        std::io::ErrorKind::DirectoryNotEmpty => {
            t!("io_error.directory_not_empty", raw_error = error)
        }
        std::io::ErrorKind::ReadOnlyFilesystem => {
            t!("io_error.read_only_filesystem", raw_error = error)
        }
        std::io::ErrorKind::StaleNetworkFileHandle => {
            t!("io_error.stale_network_file_handle", raw_error = error)
        }
        std::io::ErrorKind::InvalidInput => t!("io_error.invalid_input", raw_error = error),
        std::io::ErrorKind::InvalidData => t!("io_error.invalid_data", raw_error = error),
        std::io::ErrorKind::TimedOut => t!("io_error.timed_out", raw_error = error),
        std::io::ErrorKind::WriteZero => t!("io_error.write_zero", raw_error = error),
        std::io::ErrorKind::StorageFull => t!("io_error.storage_full", raw_error = error),
        std::io::ErrorKind::NotSeekable => t!("io_error.not_seekable", raw_error = error),
        std::io::ErrorKind::QuotaExceeded => t!("io_error.quota_exceeded", raw_error = error),
        std::io::ErrorKind::FileTooLarge => t!("io_error.file_too_large", raw_error = error),
        std::io::ErrorKind::ResourceBusy => t!("io_error.resource_busy", raw_error = error),
        std::io::ErrorKind::ExecutableFileBusy => {
            t!("io_error.executable_file_busy", raw_error = error)
        }
        std::io::ErrorKind::Deadlock => t!("io_error.deadlock", raw_error = error),
        std::io::ErrorKind::CrossesDevices => t!("io_error.crosses_devices", raw_error = error),
        std::io::ErrorKind::TooManyLinks => t!("io_error.too_many_links", raw_error = error),
        std::io::ErrorKind::InvalidFilename => t!("io_error.invalid_filename", raw_error = error),
        std::io::ErrorKind::ArgumentListTooLong => {
            t!("io_error.argument_list_too_long", raw_error = error)
        }
        std::io::ErrorKind::Interrupted => t!("io_error.interrupted", raw_error = error),
        std::io::ErrorKind::Unsupported => t!("io_error.unsupported", raw_error = error),
        std::io::ErrorKind::UnexpectedEof => t!("io_error.unexpected_eof", raw_error = error),
        std::io::ErrorKind::OutOfMemory => t!("io_error.out_of_memory", raw_error = error),
        std::io::ErrorKind::Other => t!("io_error.other", error = error.to_string()),
        _ => t!("io_error.other", error = error.to_string()),
    }
}
