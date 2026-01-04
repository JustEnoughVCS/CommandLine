use colored::Colorize;
use just_enough_vcs::{
    data::compile_info::CoreCompileInfo,
    system::action_system::{action::ActionContext, action_pool::ActionPool},
    utils::{
        cfg_file::config::ConfigFile,
        data_struct::dada_sort::quick_sort_with_cmp,
        sha1_hash,
        string_proc::{
            self,
            format_path::{format_path, format_path_str},
            snake_case,
        },
        tcp_connection::instance::ConnectionInstance,
    },
    vcs::{
        actions::{
            local_actions::{
                SetUpstreamVaultActionResult, SyncCachedSheetFailReason, UpdateToLatestInfoResult,
                proc_update_to_latest_info_action,
            },
            sheet_actions::{
                DropSheetActionResult, EditMappingActionArguments, EditMappingActionResult,
                EditMappingOperations, InvalidMoveReason, MakeSheetActionResult, OperationArgument,
                proc_drop_sheet_action, proc_edit_mapping_action, proc_make_sheet_action,
            },
            track_action::{
                CreateTaskResult, NextVersion, SyncTaskResult, TrackFileActionArguments,
                TrackFileActionResult, UpdateDescription, UpdateTaskResult, VerifyFailReason,
                proc_track_file_action,
            },
            user_actions::{
                ChangeVirtualFileEditRightResult, EditRightChangeBehaviour,
                proc_change_virtual_file_edit_right_action,
            },
        },
        constants::{
            CLIENT_FILE_TODOLIST, CLIENT_FILE_WORKSPACE, CLIENT_FOLDER_WORKSPACE_ROOT_NAME,
            CLIENT_PATH_WORKSPACE_ROOT, PORT,
        },
        current::{correct_current_dir, current_cfg_dir, current_local_path},
        data::{
            local::{
                LocalWorkspace,
                align::{AlignTaskName, AlignTasks},
                cached_sheet::CachedSheet,
                config::LocalConfig,
                latest_file_data::LatestFileData,
                latest_info::LatestInfo,
                vault_modified::check_vault_modified,
                workspace_analyzer::{AnalyzeResult, FromRelativePathBuf},
            },
            member::{Member, MemberId},
            sheet::{SheetData, SheetMappingMetadata},
            user::UserDirectory,
            vault::virtual_file::{VirtualFileId, VirtualFileVersion},
        },
        docs::{ASCII_YIZI, document, documents},
    },
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env::{current_dir, set_current_dir},
    io::Error,
    net::SocketAddr,
    path::PathBuf,
    process::exit,
    str::FromStr,
    sync::Arc,
    time::SystemTime,
};

use clap::{Parser, Subcommand, arg, command};
use just_enough_vcs::{
    utils::tcp_connection::error::TcpTargetError,
    vcs::{actions::local_actions::proc_set_upstream_vault_action, registry::client_registry},
};
use just_enough_vcs_cli::{
    data::{
        compile_info::CompileInfo,
        ipaddress_history::{get_recent_ip_address, insert_recent_ip_address},
    },
    utils::{
        display::{SimpleTable, display_width, md, size_str},
        env::{auto_update_outdate, current_locales, enable_auto_update},
        fs::move_across_partitions,
        globber::{GlobItem, Globber},
        input::{confirm_hint, confirm_hint_or, input_with_editor, show_in_pager},
        push_version::push_version,
        socket_addr_helper,
    },
};
use rust_i18n::{set_locale, t};
use tokio::{
    fs::{self},
    net::TcpSocket,
    process::Command,
    sync::mpsc::{self, Receiver},
};

// Import i18n files
rust_i18n::i18n!("locales", fallback = "en");

#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]

struct JustEnoughVcsWorkspace {
    #[command(subcommand)]
    command: JustEnoughVcsWorkspaceCommand,
}

#[derive(Subcommand, Debug)]
enum JustEnoughVcsWorkspaceCommand {
    /// Version information
    #[command(alias = "--version", alias = "-v")]
    Version(VersionArgs),

    /// Display help information
    #[command(alias = "--help", alias = "-h")]
    Help,

    // Member management
    /// Manage your local accounts
    #[command(subcommand, alias = "acc")]
    Account(AccountManage),

    /// Create an empty workspace
    Create(CreateWorkspaceArgs),

    /// Create an empty workspace in the current directory
    Init(InitWorkspaceArgs),

    /// Get workspace information in the current directory
    #[command(alias = "h")]
    Here(HereArgs),

    /// Display current sheet status information
    #[command(alias = "s")]
    Status(StatusArgs),

    /// Display detailed information about the specified file
    Info(InfoArgs),

    // Sheet management
    /// Manage sheets in the workspace
    #[command(subcommand, alias = "sh")]
    Sheet(SheetManage),

    // File management
    /// Track files to the upstream vault
    /// First track - Create and upload the "First Version", then hold them
    /// Subsequent tracks - Update files with new versions
    #[command(alias = "t")]
    Track(TrackFileArgs),

    /// Hold files for editing
    #[command(alias = "hd")]
    Hold(HoldFileArgs),

    /// Throw files, and release edit rights
    #[command(alias = "tr")]
    Throw(ThrowFileArgs),

    /// Move or rename files safely
    #[command(alias = "mv")]
    Move(MoveMappingArgs),

    /// Share file visibility to other sheets
    Share(ShareFileArgs),

    /// Sync information from upstream vault
    #[command(alias = "u")]
    Update(UpdateArgs),

    // Connection management
    /// Direct to an upstream vault and stain this workspace
    Direct(DirectArgs),

    /// DANGER ZONE : Unstain this workspace
    Unstain(UnstainArgs),

    // Other
    /// Query built-in documentation
    Docs(DocsArgs),

    // Lazy commands
    /// Try exit current sheet
    Exit,

    /// Try exit current sheet and use another sheet
    Use(UseArgs),

    /// List all sheets
    Sheets,

    /// List all accounts
    Accounts,

    /// Align file structure
    Align(SheetAlignArgs),

    /// Set current local workspace account
    As(SetLocalWorkspaceAccountArgs),

    /// Make a new sheet
    Make(SheetMakeArgs),

    /// Drop a sheet
    Drop(SheetDropArgs),

    /// As Member, Direct, and Update
    #[command(alias = "signin")]
    Login(LoginArgs),

    // Completion Helpers
    /// Display History IP Address
    #[command(name = "_ip_history")]
    GetHistoryIpAddress,

    /// Display Workspace Directory
    #[command(name = "_workspace_dir")]
    GetWorkspaceDir,

    /// Display Current Account
    #[command(name = "_account")]
    GetCurrentAccount,

    /// Display Current Upstream Vault
    #[command(name = "_upstream")]
    GetCurrentUpstream,

    /// Display Current Sheet
    #[command(name = "_sheet")]
    GetCurrentSheet,

    // Debug Tools
    #[command(name = "_glob")]
    DebugGlob(DebugGlobArgs),
}

#[derive(Parser, Debug)]
struct VersionArgs {
    #[arg(short = 'C', long = "compile-info")]
    compile_info: bool,

    #[arg(long)]
    without_banner: bool,
}

#[derive(Subcommand, Debug)]
enum AccountManage {
    /// Show help information
    #[command(alias = "--help", alias = "-h")]
    Help,

    /// Register a member to this computer
    #[command(alias = "+")]
    Add(AccountAddArgs),

    /// Remove a account from this computer
    #[command(alias = "rm", alias = "-")]
    Remove(AccountRemoveArgs),

    /// List all accounts in this computer
    #[command(alias = "ls")]
    List(AccountListArgs),

    /// Set current local workspace account
    As(SetLocalWorkspaceAccountArgs),

    /// Move private key file to account
    #[command(alias = "mvkey", alias = "mvk", alias = "movekey")]
    MoveKey(MoveKeyToAccountArgs),

    /// Output public key file to specified directory
    #[command(alias = "genpub")]
    GeneratePublicKey(GeneratePublicKeyArgs),
}

#[derive(Subcommand, Debug)]
enum SheetManage {
    /// Show help information
    #[command(alias = "--help", alias = "-h")]
    Help,

    /// List all sheets
    #[command(alias = "ls")]
    List(SheetListArgs),

    /// Use a sheet
    Use(SheetUseArgs),

    /// Exit current sheet
    Exit(SheetExitArgs),

    /// Create a new sheet
    #[command(alias = "mk")]
    Make(SheetMakeArgs),

    /// Drop current sheet
    Drop(SheetDropArgs),

    /// Align file structure
    Align(SheetAlignArgs),
}

#[derive(Parser, Debug)]
struct SheetListArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Show other's sheets
    #[arg(short, long = "other")]
    others: bool,

    /// Show all sheets
    #[arg(short = 'A', long)]
    all: bool,

    /// Show raw output
    #[arg(short, long)]
    raw: bool,
}

#[derive(Parser, Debug)]
struct SheetUseArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Sheet name
    sheet_name: String,
}

#[derive(Parser, Debug)]
struct SheetExitArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct SheetMakeArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Sheet name
    sheet_name: String,
}

#[derive(Parser, Debug)]
struct SheetDropArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Whether to skip confirmation
    #[arg(short = 'C', long)]
    confirm: bool,

    /// Sheet name
    sheet_name: String,
}

#[derive(Parser, Debug)]
struct SheetAlignArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Align task
    task: Option<String>,

    /// Align operation
    to: Option<String>,

    /// List Option: All
    #[arg(long = "all")]
    list_all: bool,

    /// List Option: Unsolved
    #[arg(long = "unsolved")]
    list_unsolved: bool,

    /// List Option: Created
    #[arg(long = "created")]
    list_created: bool,

    /// Editor mode
    #[arg(short, long)]
    work: bool,

    /// Show raw output (for list)
    #[arg(short, long)]
    raw: bool,
}

#[derive(Parser, Debug)]
struct LoginArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Whether to skip confirmation
    #[arg(short = 'C', long)]
    confirm: bool,

    /// Member ID
    login_member_id: MemberId,

    /// Upstream
    upstream: String,
}

#[derive(Parser, Debug)]
struct CreateWorkspaceArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Workspace directory path
    path: Option<PathBuf>,

    /// Force create, ignore files in the directory
    #[arg(short, long)]
    force: bool,
}

#[derive(Parser, Debug)]
struct InitWorkspaceArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Force create, ignore files in the directory
    #[arg(short, long)]
    force: bool,
}

#[derive(Parser, Debug)]
struct HereArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Show help information
    #[arg(short = 'd', long = "desc")]
    show_description: bool,
}

#[derive(Parser, Debug)]
struct StatusArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct InfoArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// File pattern
    file_pattern: Option<String>,

    /// Full histories output
    #[arg(short, long = "full")]
    full: bool,
}

#[derive(Parser, Debug)]
struct AccountAddArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Account name
    account_name: String,

    /// Auto generate ED25519 private key
    #[arg(short = 'K', long = "keygen")]
    keygen: bool,
}

#[derive(Parser, Debug)]
struct AccountRemoveArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Account name
    account_name: String,
}

#[derive(Parser, Debug)]
struct AccountListArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Show raw output
    #[arg(short, long)]
    raw: bool,
}

#[derive(Parser, Debug)]
struct SetLocalWorkspaceAccountArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Account name
    account_name: String,
}

#[derive(Parser, Debug)]
struct MoveKeyToAccountArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Account name
    account_name: String,

    /// Private key file path
    key_path: PathBuf,
}

#[derive(Parser, Debug)]
struct GeneratePublicKeyArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Account name
    account_name: String,

    /// Private key file path
    output_dir: Option<PathBuf>,
}

#[derive(Parser, Debug, Clone)]
struct TrackFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Track file pattern
    track_file_pattern: Option<String>,

    /// Overwrite modified
    #[arg(short = 'o', long = "overwrite")]
    allow_overwrite: bool,

    /// Commit - Description
    #[arg(short, long)]
    desc: Option<String>,

    /// Commit - Description
    #[arg(short = 'v', long = "version")]
    next_version: Option<String>,

    /// Commit - Editor mode
    #[arg(short, long)]
    work: bool,
}

#[derive(Parser, Debug)]
struct HoldFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Hold file pattern
    hold_file_pattern: Option<String>,

    /// Show fail details
    #[arg(short = 'd', long = "details")]
    show_fail_details: bool,

    /// Skip failed items
    #[arg(short = 'S', long)]
    skip_failed: bool,

    /// Skip check
    #[arg(short = 'F', long)]
    force: bool,
}

#[derive(Parser, Debug)]
struct ThrowFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Throw file pattern
    throw_file_pattern: Option<String>,

    /// Show fail details
    #[arg(short = 'd', long = "details")]
    show_fail_details: bool,

    /// Skip failed items
    #[arg(short = 'S', long)]
    skip_failed: bool,

    /// Skip check
    #[arg(short = 'F', long)]
    force: bool,
}

#[derive(Parser, Debug)]
struct MoveMappingArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Move mapping pattern
    move_mapping_pattern: Option<String>,

    /// To mapping pattern
    to_mapping_pattern: Option<String>,

    /// Erase
    #[arg(short = 'e', long)]
    erase: bool,

    /// Only modify upstream mapping
    #[arg(short = 'r', long)]
    only_remote: bool,
}

#[derive(Parser, Debug)]
struct ShareFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Arguments 1
    args1: Option<String>,

    /// Arguments 2
    args2: Option<String>,

    /// Arguments 3
    args3: Option<String>,
}

#[derive(Parser, Debug)]
struct UpdateArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Silent mode
    #[arg(short, long)]
    silent: bool,
}

#[derive(Parser, Debug)]
struct DirectArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Upstream vault address
    upstream: Option<String>,

    /// Whether to skip confirmation
    #[arg(short = 'C', long)]
    confirm: bool,
}

#[derive(Parser, Debug)]
struct UnstainArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Whether to skip confirmation
    #[arg(short = 'C', long)]
    confirm: bool,
}

#[derive(Parser, Debug)]
struct DocsArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Name of the docs
    docs_name: Option<String>,

    /// Direct output
    #[arg(short, long)]
    direct: bool,

    /// Show raw output for list
    #[arg(short, long)]
    raw: bool,
}

#[derive(Parser, Debug)]
struct UseArgs {
    sheet_name: String,
}

#[derive(Parser, Debug)]
struct DebugGlobArgs {
    /// Pattern
    pattern: String, // Using 'noglob jvv _glob' in ZSH plz
}

#[tokio::main]
async fn main() {
    // Init i18n
    set_locale(&current_locales());

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    // Outdate update
    let required_outdated_minutes = auto_update_outdate();
    let outdate_update_enabled = required_outdated_minutes >= 0;

    // Auto update
    let enable_auto_update = enable_auto_update();

    // The following conditions will trigger automatic update:
    // 1. Auto-update is enabled
    // 2. Vault has been modified OR (timeout update is enabled AND timeout is set to 0)
    if enable_auto_update
        && (check_vault_modified().await
            || outdate_update_enabled && required_outdated_minutes == 0)
    {
        // Record current directory
        let path = match current_dir() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("{}", t!("jv.fail.get_current_dir", error = e.to_string()));
                return;
            }
        };
        // Update
        // This will change the current current_dir
        jv_update(UpdateArgs {
            help: false,
            silent: true,
        })
        .await;
        // Restore current directory
        if let Err(e) = set_current_dir(&path) {
            eprintln!(
                "{}",
                t!(
                    "jv.fail.std.set_current_dir",
                    dir = path.display(),
                    error = e
                )
            );
            return;
        }
    } else
    // If automatic update and timeout update are enabled,
    // but required time > 0 (not in disabled or always-update state)
    if enable_auto_update && outdate_update_enabled && required_outdated_minutes > 0 {
        // Read the last update time and calculate the duration
        if let Some(local_cfg) = LocalConfig::read().await.ok() {
            if let Some(local_dir) = current_local_path() {
                if let Ok(latest_info) = LatestInfo::read_from(LatestInfo::latest_info_path(
                    &local_dir,
                    &local_cfg.current_account(),
                ))
                .await
                {
                    if let Some(update_instant) = latest_info.update_instant {
                        let now = SystemTime::now();
                        let duration_secs = now
                            .duration_since(update_instant)
                            .unwrap_or_default()
                            .as_secs();

                        if duration_secs > required_outdated_minutes as u64 * 60 {
                            // Update
                            // This will change the current current_dir
                            jv_update(UpdateArgs {
                                help: false,
                                silent: true,
                            })
                            .await
                        }
                    }
                }
            };
        };
    }

    let Ok(parser) = JustEnoughVcsWorkspace::try_parse() else {
        eprintln!("{}", md(t!("jv.fail.parse.parser_failed")));

        // Tips
        // Guide to create
        {
            // Check if workspace exist
            let Some(local_dir) = current_local_path() else {
                println!();
                println!("{}", t!("jv.tip.not_workspace").trim().yellow());
                return;
            };

            let _ = correct_current_dir();

            // Check if account list is not empty
            let Some(dir) = UserDirectory::current_cfg_dir() else {
                return;
            };

            if let Ok(ids) = dir.account_ids() {
                if ids.len() < 1 {
                    println!();
                    println!("{}", t!("jv.tip.no_account").trim().yellow());
                    return;
                }
            }

            // Check if the workspace has a registered account (account = unknown)
            let Some(local_cfg) = LocalConfig::read().await.ok() else {
                eprintln!("{}", md(t!("jv.fail.read_cfg")));
                return;
            };

            // Account exists check
            if local_cfg.current_account() == "unknown" {
                println!();
                println!("{}", t!("jv.tip.no_account_set").trim().yellow());
            } else {
                if dir
                    .account_ids()
                    .ok()
                    .map(|ids| !ids.contains(&local_cfg.current_account()))
                    .unwrap_or(false)
                {
                    println!();
                    println!(
                        "{}",
                        t!(
                            "jv.tip.account_not_exist",
                            account = local_cfg.current_account()
                        )
                        .trim()
                        .yellow()
                    );
                    return;
                }
            }

            // Outdated
            let Ok(latest_info) = LatestInfo::read_from(LatestInfo::latest_info_path(
                &local_dir,
                &local_cfg.current_account(),
            ))
            .await
            else {
                return;
            };
            if let Some(instant) = latest_info.update_instant {
                let now = SystemTime::now();
                let duration = now.duration_since(instant).unwrap_or_default();

                if duration.as_secs() > 60 * required_outdated_minutes.clamp(5, i64::MAX) as u64 {
                    // Automatically prompt if exceeding the set timeout (at least 5 minutes)
                    let hours = duration.as_secs() / 3600;
                    let minutes = (duration.as_secs() % 3600) / 60;

                    println!(
                        "\n{}",
                        t!("jv.tip.outdated", hour = hours, minutes = minutes)
                            .trim()
                            .yellow()
                    );
                }
            }
        }

        return;
    };

    match parser.command {
        JustEnoughVcsWorkspaceCommand::Version(version_args) => {
            let compile_info = CompileInfo::default();
            let core_compile_info = CoreCompileInfo::default();
            if version_args.without_banner {
                println!(
                    "{}",
                    md(t!(
                        "jv.version.header",
                        version = compile_info.cli_version,
                        vcs_version = core_compile_info.vcs_version
                    ))
                );
            } else {
                println!();
                let ascii_art_banner = ASCII_YIZI
                    .split('\n')
                    .skip_while(|line| !line.contains("#BANNER START#"))
                    .skip(1)
                    .take_while(|line| !line.contains("#BANNER END#"))
                    .collect::<Vec<&str>>()
                    .join("\n");

                println!(
                    "{}",
                    ascii_art_banner
                        .replace("{banner_line_1}", "JustEnoughVCS")
                        .replace(
                            "{banner_line_2}",
                            &format!(
                                "{}: {} ({})",
                                t!("common.word.cli_version"),
                                &compile_info.cli_version,
                                &compile_info.date
                            )
                        )
                        .replace(
                            "{banner_line_3}",
                            &format!(
                                "{}: {}",
                                t!("common.word.vcs_version"),
                                &core_compile_info.vcs_version
                            )
                        )
                );

                if !version_args.compile_info {
                    println!();
                }
            }

            if version_args.compile_info {
                println!(
                    "\n{}",
                    md(t!(
                        "jv.version.compile_info",
                        build_time = compile_info.date,
                        build_target = compile_info.target,
                        build_platform = compile_info.platform,
                        build_toolchain = compile_info.toolchain,
                        cli_build_branch = compile_info.build_branch,
                        cli_build_commit =
                            &compile_info.build_commit[..7.min(compile_info.build_commit.len())],
                        core_build_branch = core_compile_info.build_branch,
                        core_build_commit = &core_compile_info.build_commit
                            [..7.min(core_compile_info.build_commit.len())]
                    ))
                );
            }
        }

        JustEnoughVcsWorkspaceCommand::Help => {
            println!("{}", md(t!("jv.help")));
        }

        JustEnoughVcsWorkspaceCommand::Account(account_manage) => {
            let user_dir = match UserDirectory::current_cfg_dir() {
                Some(dir) => dir,
                None => {
                    eprintln!("{}", t!("jv.fail.account.no_user_dir"));
                    return;
                }
            };

            match account_manage {
                AccountManage::Help => {
                    println!("{}", md(t!("jv.account")));
                }
                AccountManage::Add(account_add_args) => {
                    if account_add_args.help {
                        println!("{}", md(t!("jv.account")));
                        return;
                    }
                    jv_account_add(user_dir, account_add_args).await;
                }
                AccountManage::Remove(account_remove_args) => {
                    if account_remove_args.help {
                        println!("{}", md(t!("jv.account")));
                        return;
                    }
                    jv_account_remove(user_dir, account_remove_args).await;
                }
                AccountManage::List(account_list_args) => {
                    if account_list_args.help {
                        println!("{}", md(t!("jv.account")));
                        return;
                    }
                    jv_account_list(user_dir, account_list_args).await;
                }
                AccountManage::As(set_local_workspace_account_args) => {
                    if set_local_workspace_account_args.help {
                        println!("{}", md(t!("jv.account")));
                        return;
                    }
                    jv_account_as(user_dir, set_local_workspace_account_args).await;
                }
                AccountManage::MoveKey(move_key_to_account_args) => {
                    if move_key_to_account_args.help {
                        println!("{}", md(t!("jv.account")));
                        return;
                    }
                    jv_account_move_key(user_dir, move_key_to_account_args).await;
                }
                AccountManage::GeneratePublicKey(generate_public_key_args) => {
                    if generate_public_key_args.help {
                        println!("{}", md(t!("jv.account")));
                        return;
                    }
                    jv_account_generate_pub_key(user_dir, generate_public_key_args).await;
                }
            }
        }
        JustEnoughVcsWorkspaceCommand::Create(create_workspace_args) => {
            if create_workspace_args.help {
                println!("{}", md(t!("jv.create")));
                return;
            }
            jv_create(create_workspace_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Init(init_workspace_args) => {
            if init_workspace_args.help {
                println!("{}", md(t!("jv.init")));
                return;
            }
            jv_init(init_workspace_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Here(here_args) => {
            if here_args.help {
                println!("{}", md(t!("jv.here")));
                return;
            }
            jv_here(here_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Status(status_args) => {
            if status_args.help {
                println!("{}", md(t!("jv.status")));
                return;
            }
            jv_status(status_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Info(info_args) => {
            if info_args.help {
                println!("{}", md(t!("jv.info")));
                return;
            }
            jv_info(info_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Sheet(sheet_manage) => match sheet_manage {
            SheetManage::Help => {
                println!("{}", md(t!("jv.sheet")));
                return;
            }
            SheetManage::List(sheet_list_args) => jv_sheet_list(sheet_list_args).await,
            SheetManage::Use(sheet_use_args) => jv_sheet_use(sheet_use_args).await,
            SheetManage::Exit(sheet_exit_args) => {
                let _ = jv_sheet_exit(sheet_exit_args).await;
            }
            SheetManage::Make(sheet_make_args) => jv_sheet_make(sheet_make_args).await,
            SheetManage::Drop(sheet_drop_args) => jv_sheet_drop(sheet_drop_args).await,
            SheetManage::Align(sheet_align_args) => jv_sheet_align(sheet_align_args).await,
        },
        JustEnoughVcsWorkspaceCommand::Track(track_file_args) => {
            if track_file_args.help {
                println!("{}", md(t!("jv.track")));
                return;
            }
            jv_track(track_file_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Hold(hold_file_args) => {
            if hold_file_args.help {
                println!("{}", md(t!("jv.hold")));
                return;
            }
            jv_hold(hold_file_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Throw(throw_file_args) => {
            if throw_file_args.help {
                println!("{}", md(t!("jv.throw")));
                return;
            }
            jv_throw(throw_file_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Move(move_file_args) => {
            if move_file_args.help {
                println!("{}", md(t!("jv.move")));
                return;
            }
            jv_move(move_file_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Share(share_file_args) => {
            if share_file_args.help {
                println!("{}", md(t!("jv.share")));
                return;
            }
            jv_share(share_file_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Update(update_file_args) => {
            if update_file_args.help {
                println!("{}", md(t!("jv.update")));
                return;
            }
            jv_update(update_file_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Direct(direct_args) => {
            if direct_args.help {
                println!("{}", md(t!("jv.direct")));
                return;
            }
            jv_direct(direct_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Unstain(unstain_args) => {
            if unstain_args.help {
                println!("{}", md(t!("jv.unstain")));
                return;
            }
            jv_unstain(unstain_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Docs(docs_args) => {
            if docs_args.help {
                println!("{}", md(t!("jv.docs")));
                return;
            }
            jv_docs(docs_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Exit => {
            let _ = jv_sheet_exit(SheetExitArgs { help: false }).await;
        }
        JustEnoughVcsWorkspaceCommand::Use(use_args) => {
            if let Ok(_) = jv_sheet_exit(SheetExitArgs { help: false }).await {
                jv_sheet_use(SheetUseArgs {
                    help: false,
                    sheet_name: use_args.sheet_name,
                })
                .await;
            }
        }
        JustEnoughVcsWorkspaceCommand::Sheets => {
            jv_sheet_list(SheetListArgs {
                help: false,
                others: false,
                all: false,
                raw: false,
            })
            .await
        }
        JustEnoughVcsWorkspaceCommand::Accounts => {
            let user_dir = match UserDirectory::current_cfg_dir() {
                Some(dir) => dir,
                None => {
                    eprintln!("{}", t!("jv.fail.account.no_user_dir"));
                    return;
                }
            };
            jv_account_list(
                user_dir,
                AccountListArgs {
                    help: false,
                    raw: false,
                },
            )
            .await
        }
        JustEnoughVcsWorkspaceCommand::Align(sheet_align_args) => {
            if sheet_align_args.help {
                println!("{}", md(t!("jv.align")));
                return;
            }
            jv_sheet_align(sheet_align_args).await
        }
        JustEnoughVcsWorkspaceCommand::As(args) => {
            let user_dir = match UserDirectory::current_cfg_dir() {
                Some(dir) => dir,
                None => {
                    eprintln!("{}", t!("jv.fail.account.no_user_dir"));
                    return;
                }
            };
            jv_account_as(user_dir, args).await
        }
        JustEnoughVcsWorkspaceCommand::Make(args) => {
            jv_sheet_make(args).await;
        }
        JustEnoughVcsWorkspaceCommand::Drop(args) => {
            jv_sheet_drop(args).await;
        }
        JustEnoughVcsWorkspaceCommand::Login(args) => {
            if !args.confirm {
                println!(
                    "{}",
                    t!(
                        "jv.confirm.login",
                        account = args.login_member_id,
                        upstream = args.upstream
                    )
                    .trim()
                    .yellow()
                );
                confirm_hint_or(t!("common.confirm"), || exit(1)).await;
            }

            let user_dir = match UserDirectory::current_cfg_dir() {
                Some(dir) => dir,
                None => {
                    eprintln!("{}", t!("jv.fail.account.no_user_dir"));
                    return;
                }
            };

            jv_account_as(
                user_dir,
                SetLocalWorkspaceAccountArgs {
                    help: false,
                    account_name: args.login_member_id,
                },
            )
            .await;

            jv_direct(DirectArgs {
                help: false,
                upstream: Some(args.upstream.clone()),
                confirm: true,
            })
            .await;

            jv_update(UpdateArgs {
                help: false,
                silent: true,
            })
            .await;

            if let Some(local_dir) = current_local_path() {
                let _ = fs::remove_file(local_dir.join(CLIENT_FILE_TODOLIST)).await;
            };
        }

        // Completion Helpers
        JustEnoughVcsWorkspaceCommand::GetHistoryIpAddress => {
            get_recent_ip_address()
                .await
                .iter()
                .for_each(|ip| println!("{}", ip));
        }
        JustEnoughVcsWorkspaceCommand::GetWorkspaceDir => {
            if let Some(local_dir) = current_local_path() {
                println!("{}", local_dir.display());
                return;
            };
            exit(1)
        }
        JustEnoughVcsWorkspaceCommand::GetCurrentAccount => {
            let _ = correct_current_dir();
            if let Ok(local_config) = LocalConfig::read().await {
                if local_config.is_host_mode() {
                    println!("host/{}", local_config.current_account());
                    return;
                } else {
                    println!("{}", local_config.current_account());
                    return;
                }
            };
            exit(1)
        }
        JustEnoughVcsWorkspaceCommand::GetCurrentUpstream => {
            let _ = correct_current_dir();
            if let Ok(local_config) = LocalConfig::read().await {
                println!("{}", local_config.upstream_addr());
                return;
            };
            exit(1)
        }
        JustEnoughVcsWorkspaceCommand::GetCurrentSheet => {
            let _ = correct_current_dir();
            if let Ok(local_config) = LocalConfig::read().await {
                let sheet_name = local_config.sheet_in_use().clone().unwrap_or_default();
                if sheet_name.len() > 0 {
                    println!("{}", sheet_name);
                    return;
                }
            };
            exit(1)
        }

        // Debug Tools
        JustEnoughVcsWorkspaceCommand::DebugGlob(glob_args) => {
            jv_debug_glob(glob_args).await;
        }
    }
}

async fn jv_create(args: CreateWorkspaceArgs) {
    let Some(path) = args.path else {
        println!("{}", md(t!("jv.create")));
        return;
    };

    if !args.force && path.exists() && !is_directory_empty(&path).await {
        eprintln!("{}", t!("jv.fail.init_create_dir_not_empty").trim());
        return;
    }

    match LocalWorkspace::setup_local_workspace(path).await {
        Ok(_) => {
            println!("{}", t!("jv.success.create"));
        }
        Err(e) => {
            eprintln!("{}", t!("jv.fail.create", error = e.to_string()));
        }
    }
}

async fn jv_init(args: InitWorkspaceArgs) {
    let path = match current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("{}", t!("jv.fail.get_current_dir", error = e.to_string()));
            return;
        }
    };

    if !args.force && path.exists() && !is_directory_empty(&path).await {
        eprintln!("{}", t!("jv.fail.init_create_dir_not_empty").trim());
        return;
    }

    match LocalWorkspace::setup_local_workspace(path).await {
        Ok(_) => {
            println!("{}", t!("jv.success.init"));
        }
        Err(e) => {
            eprintln!("{}", t!("jv.fail.init", error = e.to_string()));
        }
    }
}

async fn is_directory_empty(path: &PathBuf) -> bool {
    match fs::read_dir(path).await {
        Ok(mut entries) => entries.next_entry().await.unwrap().is_none(),
        Err(_) => false,
    }
}

async fn jv_here(args: HereArgs) {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Ok(local_cfg) = LocalConfig::read_from(local_dir.join(CLIENT_FILE_WORKSPACE)).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(latest_info) = LatestInfo::read_from(LatestInfo::latest_info_path(
        &local_dir,
        &local_cfg.current_account(),
    ))
    .await
    else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(latest_file_data_path) = LatestFileData::data_path(&local_cfg.current_account()) else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(latest_file_data) = LatestFileData::read_from(&latest_file_data_path).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    // Print path information
    let sheet_name = if let Some(sheet_name) = local_cfg.sheet_in_use() {
        sheet_name.to_string()
    } else {
        "".to_string()
    };

    // Read cached sheet
    let Ok(cached_sheet) = CachedSheet::cached_sheet_data(&sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_cfg.clone()) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    let Ok(analyzed) = AnalyzeResult::analyze_local_status(&local_workspace).await else {
        eprintln!("{}", md(t!("jv.fail.status.analyze")).trim());
        return;
    };

    // Read local sheet
    let Ok(local_sheet) = local_workspace
        .local_sheet(&local_cfg.current_account(), &sheet_name)
        .await
    else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let path = match current_dir() {
        Ok(path) => path,
        Err(_) => {
            eprintln!("{}", t!("jv.fail.get_current_dir"));
            return;
        }
    };

    let local_dir = match current_local_path() {
        Some(path) => path,
        None => {
            eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
            return;
        }
    };

    let relative_path = match path.strip_prefix(&local_dir) {
        Ok(path) => path.display().to_string(),
        Err(_) => path.display().to_string(),
    };

    let mut remote_files = mapping_names_here(&path, &local_dir, &cached_sheet);

    let duration_updated = SystemTime::now()
        .duration_since(latest_info.update_instant.unwrap_or(SystemTime::now()))
        .unwrap_or_default();
    let minutes = duration_updated.as_secs() / 60;

    let account_str = if local_cfg.is_host_mode() {
        format!("{}/{}", "host".red(), local_cfg.current_account())
    } else {
        local_cfg.current_account()
    };

    println!(
        "{}",
        t!(
            "jv.success.here.path_info",
            upstream = local_cfg.upstream_addr().to_string(),
            account = account_str,
            sheet_name = sheet_name.yellow(),
            path = relative_path,
            minutes = minutes
        )
        .trim()
    );

    // Print file info
    let mut columns = vec![
        t!("jv.success.here.items.editing"),
        t!("jv.success.here.items.holder"),
        t!("jv.success.here.items.size"),
        t!("jv.success.here.items.version"),
        t!("jv.success.here.items.name"),
    ];
    if args.show_description {
        columns.push(t!("jv.success.here.items.description"));
    }
    let mut table = SimpleTable::new(columns);

    let mut dir_count = 0;
    let mut file_count = 0;
    let mut total_size = 0;

    // Exists files
    if let Ok(mut entries) = fs::read_dir(&path).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_type) = entry.file_type().await {
                let file_name = entry.file_name().to_string_lossy().to_string();

                if file_name == CLIENT_FOLDER_WORKSPACE_ROOT_NAME {
                    continue;
                }

                let metadata = match entry.metadata().await {
                    Ok(meta) => meta,
                    Err(_) => continue,
                };

                let size = metadata.len();
                let is_dir = file_type.is_dir();
                let mut version = "-".to_string();
                let mut hold = "-".to_string();
                let mut editing = "-".to_string();
                let mut desc = "-".to_string();

                if is_dir {
                    // Directory
                    // Add directory count
                    dir_count += 1;

                    let dir_name = format!("{}/", file_name);

                    // Remove remote dirs items that already exist locally
                    remote_files.remove(&dir_name);

                    // Add directory item
                    let mut line = vec![
                        editing.to_string(),
                        hold.to_string(),
                        "-".to_string(),
                        version.to_string(),
                        t!(
                            "jv.success.here.append_info.name",
                            name = dir_name.to_string().cyan()
                        )
                        .trim()
                        .to_string(),
                    ];
                    if args.show_description {
                        line.push(desc);
                    }
                    table.insert_item(0, line);
                } else {
                    // Local File
                    // Add file count
                    file_count += 1;

                    // Get current path
                    let current_path = PathBuf::from_str(relative_path.as_ref())
                        .unwrap()
                        .join(file_name.clone());

                    // Get mapping
                    if let Some(mapping) = cached_sheet.mapping().get(&current_path) {
                        let mut is_file_held = false;
                        let mut is_version_match = false;

                        // Hold status
                        let id = mapping.id.clone();
                        if let Some(holder) = latest_file_data.file_holder(&id) {
                            if holder == &local_cfg.current_account() {
                                hold = t!("jv.success.here.append_info.holder.yourself")
                                    .trim()
                                    .green()
                                    .to_string();
                                is_file_held = true;
                            } else {
                                let holder_text = t!(
                                    "jv.success.here.append_info.holder.others",
                                    holder = holder
                                )
                                .trim()
                                .truecolor(128, 128, 128);
                                hold = holder_text.to_string();
                            }
                        }

                        // Version status && description
                        if let Some(latest_version) = latest_file_data.file_version(&id) {
                            let local_version = local_sheet.mapping_data(&current_path);
                            if let Ok(local_mapping) = local_version {
                                let local_version = local_mapping.version_when_updated();

                                // Append version status
                                if latest_version == local_version {
                                    version = t!(
                                        "jv.success.here.append_info.version.match",
                                        version = latest_version
                                    )
                                    .trim()
                                    .to_string();
                                    is_version_match = true;
                                } else {
                                    version = t!(
                                        "jv.success.here.append_info.version.unmatch",
                                        remote_version = local_version,
                                    )
                                    .trim()
                                    .red()
                                    .to_string();
                                }

                                // Append description
                                if args.show_description {
                                    let content = local_mapping
                                        .version_desc_when_updated()
                                        .description
                                        .clone();

                                    let content = truncate_first_line(content);

                                    desc = t!(
                                        "jv.success.here.append_info.description",
                                        creator = local_mapping
                                            .version_desc_when_updated()
                                            .creator
                                            .cyan()
                                            .to_string(),
                                        description = content,
                                    )
                                    .trim()
                                    .to_string();
                                }
                            }
                        }

                        // Editing status
                        let modified = analyzed.modified.contains(&current_path);
                        if !is_file_held || !is_version_match {
                            if modified {
                                editing = t!(
                                    "jv.success.here.append_info.editing.cant_edit_but_modified"
                                )
                                .trim()
                                .red()
                                .to_string();
                            } else {
                                editing = t!("jv.success.here.append_info.editing.cant_edit")
                                    .trim()
                                    .truecolor(128, 128, 128)
                                    .to_string();
                            }
                        } else {
                            if modified {
                                editing = t!("jv.success.here.append_info.editing.modified")
                                    .trim()
                                    .cyan()
                                    .to_string();
                            } else {
                                editing = t!("jv.success.here.append_info.editing.can_edit")
                                    .trim()
                                    .green()
                                    .to_string();
                            }
                        }
                    }

                    // Remove remote file items that already exist locally
                    remote_files.remove(&file_name);

                    // Add Table Item
                    let mut line = vec![
                        editing.to_string(),
                        hold.to_string(),
                        t!(
                            "jv.success.here.append_info.size",
                            size = size_str(size as usize)
                        )
                        .trim()
                        .yellow()
                        .to_string(),
                        version.to_string(),
                        t!("jv.success.here.append_info.name", name = file_name)
                            .trim()
                            .to_string(),
                    ];

                    if args.show_description {
                        line.push(desc);
                    }

                    table.push_item(line);
                }

                // Total Size
                total_size += size;
            }
        }
    }

    // Remote Files
    for mapping in remote_files {
        if let Some(metadata) = mapping.1 {
            let mut hold = "-".to_string();
            if let Some(holder) = latest_file_data.file_holder(&metadata.id) {
                if holder == &local_cfg.current_account() {
                    hold = t!("jv.success.here.append_info.holder.yourself")
                        .trim()
                        .green()
                        .to_string();
                } else {
                    let holder_text =
                        t!("jv.success.here.append_info.holder.others", holder = holder)
                            .trim()
                            .truecolor(128, 128, 128);
                    hold = holder_text.to_string();
                }
            }

            // File
            let mut line = vec![
                t!("jv.success.here.append_info.editing.not_local")
                    .trim()
                    .truecolor(128, 128, 128)
                    .to_string(),
                hold.to_string(),
                "-".to_string(),
                metadata.version,
                t!("jv.success.here.append_info.name", name = mapping.0)
                    .trim()
                    .truecolor(128, 128, 128)
                    .to_string(),
            ];

            if args.show_description {
                line.push("-".to_string());
            }

            table.push_item(line);
        } else {
            // Directory
            let mut line = vec![
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
                t!("jv.success.here.append_info.name", name = mapping.0)
                    .trim()
                    .truecolor(128, 128, 128)
                    .to_string(),
            ];

            if args.show_description {
                line.push("-".to_string());
            }

            table.push_item(line);
        }
    }

    println!("{}", table);

    // Print directory info
    println!(
        "{}",
        t!(
            "jv.success.here.count_info",
            dir_count = dir_count,
            file_count = file_count,
            size = size_str(total_size as usize)
        )
        .trim()
    );
}

async fn jv_status(_args: StatusArgs) {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    let Ok(local_cfg) = LocalConfig::read_from(local_dir.join(CLIENT_FILE_WORKSPACE)).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(latest_info) = LatestInfo::read_from(LatestInfo::latest_info_path(
        &local_dir,
        &local_cfg.current_account(),
    ))
    .await
    else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let account = local_cfg.current_account();

    let Ok(latest_file_data_path) = LatestFileData::data_path(&account) else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(latest_file_data) = LatestFileData::read_from(&latest_file_data_path).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Some(sheet_name) = local_cfg.sheet_in_use().clone() else {
        eprintln!("{}", md(t!("jv.fail.status.no_sheet_in_use")).trim());
        return;
    };

    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_cfg.clone()) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    let Ok(local_sheet) = local_workspace.local_sheet(&account, &sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let in_ref_sheet = latest_info.reference_sheets.contains(&sheet_name);
    let is_host_mode = local_cfg.is_host_mode();

    let Ok(analyzed) = AnalyzeResult::analyze_local_status(&local_workspace).await else {
        eprintln!("{}", md(t!("jv.fail.status.analyze")).trim());
        return;
    };

    // Format created items
    let mut created_items: Vec<String> = analyzed
        .created
        .iter()
        .map(|path| {
            t!(
                "jv.success.status.created_item",
                path = path.display().to_string()
            )
            .trim()
            .green()
            .to_string()
        })
        .collect();

    // Format erased items
    let mut erased_items: Vec<String> = analyzed
        .erased
        .iter()
        .map(|path| {
            t!(
                "jv.success.status.erased_item",
                path = path.display().to_string()
            )
            .trim()
            .magenta()
            .to_string()
        })
        .collect();

    // Format lost items
    let mut lost_items: Vec<String> = analyzed
        .lost
        .iter()
        .map(|path| {
            t!(
                "jv.success.status.lost_item",
                path = path.display().to_string()
            )
            .trim()
            .red()
            .to_string()
        })
        .collect();

    // Format moved items
    let mut moved_items: Vec<String> = analyzed
        .moved
        .iter()
        .map(|(_, (from, to))| {
            t!(
                "jv.success.status.moved_item",
                from = from.display(),
                to = to.display()
            )
            .trim()
            .yellow()
            .to_string()
        })
        .collect();

    // Format modified items
    let mut modified_items: Vec<String> = analyzed
        .modified
        .iter()
        .map(|path| {
            let holder_match = {
                if let Ok(mapping) = local_sheet.mapping_data(path) {
                    let vfid = mapping.mapping_vfid();
                    match latest_file_data.file_holder(vfid) {
                        Some(holder) => holder == &account,
                        None => false,
                    }
                } else {
                    false
                }
            };

            let base_version_match = {
                if let Ok(mapping) = local_sheet.mapping_data(path) {
                    let vfid = mapping.mapping_vfid();
                    let ver = mapping.version_when_updated();
                    if let Some(latest_version) = latest_file_data.file_version(&vfid) {
                        ver == latest_version
                    } else {
                        true
                    }
                } else {
                    true
                }
            };

            // Holder dismatch
            if !holder_match {
                return t!(
                    "jv.success.status.invalid_modified_item",
                    path = path.display().to_string(),
                    reason = t!("jv.success.status.invalid_modified_reasons.not_holder")
                )
                .trim()
                .red()
                .to_string();
            }

            // Base version mismatch
            if !base_version_match {
                return t!(
                    "jv.success.status.invalid_modified_item",
                    path = path.display().to_string(),
                    reason = t!("jv.success.status.invalid_modified_reasons.base_version_mismatch")
                )
                .trim()
                .red()
                .to_string();
            }

            t!(
                "jv.success.status.modified_item",
                path = path.display().to_string()
            )
            .trim()
            .cyan()
            .to_string()
        })
        .collect();

    let has_struct_changes = !created_items.is_empty()
        || !lost_items.is_empty()
        || !erased_items.is_empty()
        || !moved_items.is_empty();
    let has_file_modifications = !modified_items.is_empty();

    if has_struct_changes {
        sort_paths(&mut created_items);
        sort_paths(&mut lost_items);
        sort_paths(&mut erased_items);
        sort_paths(&mut moved_items);
    }
    if has_file_modifications {
        sort_paths(&mut modified_items);
    }

    // Calculate duration since last update
    let update_instant = latest_info.update_instant.unwrap_or(SystemTime::now());
    let duration = SystemTime::now()
        .duration_since(update_instant)
        .unwrap_or_default();
    let h = duration.as_secs() / 3600;
    let m = (duration.as_secs() % 3600) / 60;
    let s = duration.as_secs() % 60;

    if has_struct_changes {
        println!(
            "{}",
            md(t!(
                "jv.success.status.struct_changes_display",
                sheet_name = sheet_name,
                moved_items = if moved_items.is_empty() {
                    "".to_string()
                } else {
                    moved_items.join("\n") + "\n"
                },
                lost_items = if lost_items.is_empty() {
                    "".to_string()
                } else {
                    lost_items.join("\n") + "\n"
                },
                erased_items = if erased_items.is_empty() {
                    "".to_string()
                } else {
                    erased_items.join("\n") + "\n"
                },
                created_items = if created_items.is_empty() {
                    "".to_string()
                } else {
                    created_items.join("\n") + "\n"
                },
                h = h,
                m = m,
                s = s
            ))
            .trim()
        );
    } else if has_file_modifications {
        println!(
            "{}",
            md(t!(
                "jv.success.status.content_modifies_display",
                sheet_name = sheet_name,
                modified_items = if modified_items.is_empty() {
                    "".to_string()
                } else {
                    modified_items.join("\n")
                },
                h = h,
                m = m,
                s = s
            ))
            .trim()
        );
    } else {
        if in_ref_sheet {
            println!(
                "{}",
                md(t!(
                    "jv.success.status.no_changes_in_reference_sheet",
                    sheet_name = sheet_name,
                    h = h,
                    m = m,
                    s = s
                ))
            );
        } else {
            println!(
                "{}",
                md(t!(
                    "jv.success.status.no_changes",
                    sheet_name = sheet_name,
                    h = h,
                    m = m,
                    s = s
                ))
            );
        }
    }

    if in_ref_sheet && !is_host_mode {
        println!(
            "\n{}",
            md(t!("jv.success.status.hint_in_reference_sheet")).yellow()
        );
    }
    if is_host_mode {
        println!("\n{}", md(t!("jv.success.status.hint_as_host")));
    }
}

async fn jv_info(args: InfoArgs) {
    let local_dir = match current_local_path() {
        Some(dir) => dir,
        None => {
            eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
            return;
        }
    };

    let query_file_paths = if let Some(pattern) = args.file_pattern.clone() {
        let files = glob(pattern, &local_dir).await;
        files
            .iter()
            .filter_map(|f| PathBuf::from_str(f.0).ok())
            .collect::<Vec<_>>()
    } else {
        println!("{}", md(t!("jv.info")));
        return;
    };

    let _ = correct_current_dir();

    let Ok(local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(latest_info) = LatestInfo::read_from(LatestInfo::latest_info_path(
        &local_dir,
        &local_cfg.current_account(),
    ))
    .await
    else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let account = local_cfg.current_account();

    let Ok(latest_file_data_path) = LatestFileData::data_path(&account) else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    // Get latest file data
    let Ok(latest_file_data) = LatestFileData::read_from(&latest_file_data_path).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Some(sheet_name) = local_cfg.sheet_in_use().clone() else {
        eprintln!("{}", md(t!("jv.fail.status.no_sheet_in_use")).trim());
        return;
    };

    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_cfg.clone()) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    let Ok(local_sheet) = local_workspace.local_sheet(&account, &sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    if query_file_paths.len() < 1 {
        return;
    }
    // File to query
    let query_file_path = query_file_paths[0].to_path_buf();
    let Ok(mapping) = local_sheet.mapping_data(&query_file_path) else {
        return;
    };
    let vfid = mapping.mapping_vfid();

    // Render initial location
    {
        println!("{}", query_file_path.display().to_string());
    }

    // Render reference sheet location, use ID if not found
    {
        let path_in_ref = if let Some(path) = latest_info.ref_sheet_vfs_mapping.get(vfid) {
            path.display().to_string()
        } else {
            vfid.clone()
        };

        // Offset string
        let query_file_path_string = query_file_path.display().to_string();
        let offset_string = " ".repeat(display_width(
            if let Some(last_slash) = query_file_path_string.rfind('/') {
                &query_file_path_string[..last_slash]
            } else {
                ""
            },
        ));

        println!(
            "{}{}{}",
            offset_string,
            "\\_ ".truecolor(128, 128, 128),
            path_in_ref.cyan()
        );
    }

    // Render complete file history
    {
        if let Some(histories) = latest_file_data.file_histories(vfid) {
            // Get file version in reference sheet
            let version_in_ref = if let Some(mapping) = latest_info
                .ref_sheet_content
                .mapping()
                .get(&query_file_path)
            {
                mapping.version.clone()
            } else {
                "".to_string()
            };

            // Get current file version
            let version_current = latest_file_data
                .file_version(vfid)
                .cloned()
                .unwrap_or_else(|| "".to_string());

            // Check if file is being edited based on latest version (regardless of hold status)
            let modified_correctly = if let Ok(mapping) = local_sheet.mapping_data(&query_file_path)
            {
                // If base editing version is correct
                if mapping.version_when_updated() == &version_current {
                    mapping.last_modifiy_check_result() // Return detection result
                } else {
                    false
                }
            } else {
                false
            };

            // Text
            let (prefix_str, version_str, creator_str, description_str) = (
                t!("jv.success.info.oneline.table_headers.prefix"),
                t!("jv.success.info.oneline.table_headers.version"),
                t!("jv.success.info.oneline.table_headers.creator"),
                t!("jv.success.info.oneline.table_headers.description"),
            );

            // Single-line output
            if !args.full {
                // Create table
                let mut table =
                    SimpleTable::new(vec![prefix_str, version_str, creator_str, description_str]);

                // Append data
                for (version, description) in histories {
                    // If it's reference version, render "@"
                    // Current version, render "\_"
                    // Other versions, render "|"
                    let prefix = if version == &version_in_ref {
                        "@".cyan().to_string()
                    } else if version == &version_current {
                        "|->".yellow().to_string()
                    } else {
                        "|".truecolor(128, 128, 128).to_string()
                    };

                    table.insert_item(
                        0,
                        vec![
                            prefix,
                            version.to_string(),
                            format!("@{}: ", &description.creator.cyan()),
                            truncate_first_line(description.description.to_string()),
                        ],
                    );
                }

                // If file has new version, append
                if modified_correctly {
                    table.insert_item(
                        0,
                        vec![
                            "+".green().to_string(),
                            "CURRENT".green().to_string(),
                            format!(
                                "@{}: ",
                                local_workspace
                                    .config()
                                    .lock()
                                    .await
                                    .current_account()
                                    .cyan()
                            ),
                            format!(
                                "{}",
                                t!("jv.success.info.oneline.description_current").green()
                            ),
                        ],
                    );
                }

                // Render table
                let table_str = table.to_string();
                if table_str.lines().count() > 1 {
                    println!();
                }
                for line in table_str.lines().skip(1) {
                    println!("{}", line);
                }
            } else {
                // Multi-line output
                if histories.len() > 0 {
                    println!();
                }
                for (version, description) in histories {
                    println!("{}: {}", version_str, version);
                    println!("{}: {}", creator_str, description.creator.cyan());
                    println!("{}", description.description);
                    if version != &histories.last().unwrap().0 {
                        println!("{}", "-".repeat(45));
                    }
                }
            }
        }
    }
}

async fn jv_sheet_list(args: SheetListArgs) {
    let _ = correct_current_dir();

    let Some(local_dir) = current_local_path() else {
        if !args.raw {
            eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        }
        return;
    };

    let Ok(local_cfg) = LocalConfig::read().await else {
        if !args.raw {
            eprintln!("{}", md(t!("jv.fail.read_cfg")));
        }
        return;
    };

    let Ok(latest_info) = LatestInfo::read_from(LatestInfo::latest_info_path(
        &local_dir,
        &local_cfg.current_account(),
    ))
    .await
    else {
        if !args.raw {
            eprintln!("{}", md(t!("jv.fail.read_cfg")));
        }
        return;
    };

    let mut your_sheet_counts = 0;
    let mut other_sheet_counts = 0;

    if args.raw {
        // Print your sheets
        if !args.others && !args.all || !args.others {
            latest_info
                .visible_sheets
                .iter()
                .for_each(|s| println!("{}", s));
        }
        // Print other sheets
        if args.others || args.all {
            latest_info
                .invisible_sheets
                .iter()
                .for_each(|s| println!("{}", s.sheet_name));
        }
    } else {
        // Print your sheets
        if !args.others && !args.all || !args.others {
            println!("{}", md(t!("jv.success.sheet.list.your_sheet")));
            let in_use = local_cfg.sheet_in_use();
            for sheet in latest_info.visible_sheets {
                let is_ref_sheet = latest_info.reference_sheets.contains(&sheet);
                let display_name = if is_ref_sheet {
                    format!(
                        "{} {}",
                        sheet,
                        md(t!("jv.success.sheet.list.reference_sheet_suffix"))
                            .truecolor(128, 128, 128)
                    )
                } else {
                    sheet.clone()
                };

                if let Some(in_use) = in_use
                    && in_use == &sheet
                {
                    println!(
                        "{}",
                        md(t!(
                            "jv.success.sheet.list.your_sheet_item_use",
                            number = your_sheet_counts + 1,
                            name = display_name.cyan()
                        ))
                    );
                } else {
                    println!(
                        "{}",
                        md(t!(
                            "jv.success.sheet.list.your_sheet_item",
                            number = your_sheet_counts + 1,
                            name = display_name
                        ))
                    );
                }
                your_sheet_counts += 1;
            }
        }

        // Print other sheets
        if args.others || args.all {
            if args.all {
                println!();
            }
            println!("{}", md(t!("jv.success.sheet.list.other_sheet")));
            for sheet in latest_info.invisible_sheets {
                if let Some(holder) = sheet.holder_name {
                    println!(
                        "{}",
                        md(t!(
                            "jv.success.sheet.list.other_sheet_item",
                            number = other_sheet_counts + 1,
                            name = sheet.sheet_name,
                            holder = holder
                        ))
                    );
                } else {
                    println!(
                        "{}",
                        md(t!(
                            "jv.success.sheet.list.other_sheet_item_no_holder",
                            number = other_sheet_counts + 1,
                            name = sheet.sheet_name
                        ))
                    );
                }
                other_sheet_counts += 1;
            }
        }

        // If not use any sheets, print tips
        if local_cfg.sheet_in_use().is_none() {
            println!();
            if your_sheet_counts > 0 {
                println!("{}", md(t!("jv.success.sheet.list.tip_has_sheet")));
            } else {
                println!("{}", md(t!("jv.success.sheet.list.tip_no_sheet")));
            }
        }
    }
}

async fn jv_sheet_use(args: SheetUseArgs) {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let current_dir = current_dir().unwrap();

    if local_dir != current_dir {
        eprintln!("{}", t!("jv.fail.not_root_dir").trim());
        return;
    }

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    match local_cfg.use_sheet(args.sheet_name.clone()).await {
        Ok(_) => {
            let Ok(_) = LocalConfig::write(&local_cfg).await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim());
                return;
            };

            // After successfully switching sheets, status should be automatically prompted
            jv_status(StatusArgs { help: false }).await;
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => {} // Already In Use
            std::io::ErrorKind::NotFound => {
                eprintln!(
                    "{}",
                    md(t!("jv.fail.use.sheet_not_exists", name = args.sheet_name))
                );
                return;
            }
            std::io::ErrorKind::DirectoryNotEmpty => {
                eprintln!(
                    "{}",
                    md(t!(
                        "jv.fail.use.directory_not_empty",
                        name = args.sheet_name
                    ))
                );
                return;
            }
            _ => {
                handle_err(e.into());
            }
        },
    }
}

async fn jv_sheet_exit(_args: SheetExitArgs) -> Result<(), ()> {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return Err(());
    };

    let current_dir = current_dir().unwrap();

    if local_dir != current_dir {
        eprintln!("{}", t!("jv.fail.not_root_dir").trim());
        return Err(());
    }

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return Err(());
    };

    match local_cfg.exit_sheet().await {
        Ok(_) => {
            let Ok(_) = LocalConfig::write(&local_cfg).await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim());
                return Err(());
            };
            return Ok(());
        }
        Err(e) => {
            handle_err(e.into());
            return Err(());
        }
    }
}

async fn jv_sheet_make(args: SheetMakeArgs) {
    let sheet_name = snake_case!(args.sheet_name);

    let local_config = match precheck().await {
        Some(config) => config,
        None => return,
    };

    let (pool, ctx, _output) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let latest_info = match LatestInfo::read_from(LatestInfo::latest_info_path(
        &local_dir,
        &local_config.current_account(),
    ))
    .await
    {
        Ok(info) => info,
        Err(_) => {
            eprintln!("{}", t!("jv.fail.read_cfg"));
            return;
        }
    };

    if latest_info
        .invisible_sheets
        .iter()
        .any(|sheet| sheet.sheet_name == sheet_name && sheet.holder_name.is_none())
    {
        println!(
            "{}",
            md(t!("jv.confirm.sheet.make.restore", sheet_name = sheet_name)).yellow()
        );
        if !confirm_hint(t!("common.confirm")).await {
            return;
        }
    }

    match proc_make_sheet_action(&pool, ctx, sheet_name.clone()).await {
        Ok(r) => match r {
            MakeSheetActionResult::Success => {
                println!(
                    "{}",
                    md(t!("jv.result.sheet.make.success", name = sheet_name))
                )
            }
            MakeSheetActionResult::SuccessRestore => {
                println!(
                    "{}",
                    md(t!(
                        "jv.result.sheet.make.success_restore",
                        name = sheet_name
                    ))
                )
            }
            MakeSheetActionResult::AuthorizeFailed(e) => {
                eprintln!("{}", md(t!("jv.result.common.authroize_failed", err = e)))
            }
            MakeSheetActionResult::SheetAlreadyExists => {
                eprintln!(
                    "{}",
                    md(t!(
                        "jv.result.sheet.make.sheet_already_exists",
                        name = sheet_name
                    ))
                );
            }
            MakeSheetActionResult::SheetCreationFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.make.sheet_creation_failed", err = e))
                )
            }
            MakeSheetActionResult::Unknown => todo!(),
        },
        Err(e) => handle_err(e),
    }
}

async fn jv_sheet_drop(args: SheetDropArgs) {
    let sheet_name = snake_case!(args.sheet_name);

    if !args.confirm {
        println!(
            "{}",
            t!("jv.confirm.sheet.drop", sheet_name = sheet_name)
                .trim()
                .yellow()
        );
        confirm_hint_or(t!("common.confirm"), || exit(1)).await;
    }

    let local_config = match precheck().await {
        Some(config) => config,
        None => return,
    };

    let (pool, ctx, _output) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    match proc_drop_sheet_action(&pool, ctx, sheet_name.clone()).await {
        Ok(r) => match r {
            DropSheetActionResult::Success => {
                println!(
                    "{}",
                    md(t!("jv.result.sheet.drop.success", name = sheet_name))
                )
            }
            DropSheetActionResult::SheetInUse => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.drop.sheet_in_use", name = sheet_name))
                )
            }
            DropSheetActionResult::AuthorizeFailed(e) => {
                eprintln!("{}", md(t!("jv.result.common.authroize_failed", err = e)))
            }
            DropSheetActionResult::SheetNotExists => {
                eprintln!(
                    "{}",
                    md(t!(
                        "jv.result.sheet.drop.sheet_not_exists",
                        name = sheet_name
                    ))
                )
            }
            DropSheetActionResult::SheetDropFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.drop.sheet_drop_failed", err = e))
                )
            }
            DropSheetActionResult::NoHolder => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.drop.no_holder", name = sheet_name))
                )
            }
            DropSheetActionResult::NotOwner => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.drop.not_owner", name = sheet_name))
                )
            }
            _ => {}
        },
        Err(e) => handle_err(e),
    }
}

async fn jv_sheet_align(args: SheetAlignArgs) {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    let local_cfg = match precheck().await {
        Some(config) => config,
        None => {
            return;
        }
    };

    let account = local_cfg.current_account();

    let Some(sheet_name) = local_cfg.sheet_in_use().clone() else {
        eprintln!("{}", md(t!("jv.fail.status.no_sheet_in_use")).trim());
        return;
    };

    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_cfg.clone()) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    let Ok(mut local_sheet) = local_workspace.local_sheet(&account, &sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(analyzed) = AnalyzeResult::analyze_local_status(&local_workspace).await else {
        eprintln!("{}", md(t!("jv.fail.status.analyze")).trim());
        return;
    };

    let align_tasks = AlignTasks::from_analyze_result(analyzed);

    // No task input, list all tasks needs align
    let Some(task) = args.task else {
        // Raw output
        if args.raw {
            if args.list_all {
                align_tasks.created.iter().for_each(|i| println!("{}", i.0));
                align_tasks.moved.iter().for_each(|i| println!("{}", i.0));
                align_tasks.lost.iter().for_each(|i| println!("{}", i.0));
                align_tasks.erased.iter().for_each(|i| println!("{}", i.0));
                return;
            }
            if args.list_created {
                align_tasks.created.iter().for_each(|i| println!("{}", i.0));
                return;
            }
            if args.list_unsolved {
                align_tasks.moved.iter().for_each(|i| println!("{}", i.0));
                align_tasks.lost.iter().for_each(|i| println!("{}", i.0));
                align_tasks.erased.iter().for_each(|i| println!("{}", i.0));
                return;
            }
            return;
        }

        let mut table = SimpleTable::new(vec![
            t!("jv.success.sheet.align.task_name").to_string(),
            t!("jv.success.sheet.align.local_path").to_string(),
            if !align_tasks.moved.is_empty() {
                t!("jv.success.sheet.align.remote_path").to_string()
            } else {
                "".to_string()
            },
        ]);

        let mut need_align = 0;

        if !align_tasks.created.is_empty() {
            align_tasks.created.iter().for_each(|(n, p)| {
                table.push_item(vec![
                    format!("+ {}", n).green().to_string(),
                    p.display().to_string().green().to_string(),
                    "".to_string(),
                ]);
            });
        }

        if !align_tasks.lost.is_empty() {
            align_tasks.lost.iter().for_each(|(n, p)| {
                table.push_item(vec![
                    format!("- {}", n).red().to_string(),
                    p.display().to_string().red().to_string(),
                    "".to_string(),
                ]);
            });
            need_align += 1;
        }

        if !align_tasks.erased.is_empty() {
            align_tasks.erased.iter().for_each(|(n, p)| {
                table.push_item(vec![
                    format!("& {}", n).magenta().to_string(),
                    p.display().to_string().magenta().to_string(),
                    "".to_string(),
                ]);
            });
            need_align += 1;
        }

        if !align_tasks.moved.is_empty() {
            align_tasks.moved.iter().for_each(|(n, (rp, lp))| {
                table.push_item(vec![
                    format!("> {}", n).yellow().to_string(),
                    lp.display().to_string().yellow().to_string(),
                    rp.display().to_string(),
                ]);
            });
            need_align += 1;
        }

        if need_align > 0 {
            println!(
                "{}",
                md(t!("jv.success.sheet.align.list", tasks = table.to_string()))
            );

            // Suggestion1: Confirm Erased
            if align_tasks.erased.len() > 0 {
                println!(
                    "\n{}",
                    md(t!(
                        "jv.success.sheet.align.suggestion_1",
                        example_erased = align_tasks.erased[0].0
                    ))
                )
            } else
            // Suggestion2: Confirm Lost
            if align_tasks.lost.len() > 0 {
                println!(
                    "\n{}",
                    md(t!(
                        "jv.success.sheet.align.suggestion_2",
                        example_lost = align_tasks.lost[0].0
                    ))
                )
            } else
            // Suggestion3: Confirm Moved
            if align_tasks.moved.len() > 0 {
                println!(
                    "\n{}",
                    md(t!(
                        "jv.success.sheet.align.suggestion_3",
                        example_moved = align_tasks.moved[0].0
                    ))
                )
            }
        } else {
            println!("{}", md(t!("jv.success.sheet.align.no_changes").trim()));
        }

        return;
    };

    let Some(to) = args.to else {
        eprintln!("{}", md(t!("jv.fail.sheet.align.no_direction")));
        return;
    };

    // Move: alignment mode
    if task.starts_with("moved") {
        let align_to = match to.trim().to_lowercase().as_str() {
            "remote" => "remote",
            "local" => "local",
            "break" => "break",
            _ => {
                eprintln!("{}", md(t!("jv.fail.sheet.align.unknown_moved_direction")));
                return;
            }
        };

        // Build remote move operations
        let operations: HashMap<FromRelativePathBuf, OperationArgument> = if task == "moved" {
            // Align all moved items
            align_tasks
                .moved
                .iter()
                .map(|(_, (remote_path, local_path))| {
                    (
                        remote_path.clone(),
                        (EditMappingOperations::Move, Some(local_path.clone())),
                    )
                })
                .collect()
        } else {
            // Align specific moved item
            align_tasks
                .moved
                .iter()
                .filter(|(key, _)| key == &task)
                .map(|(_, (remote_path, local_path))| {
                    (
                        remote_path.clone(),
                        (EditMappingOperations::Move, Some(local_path.clone())),
                    )
                })
                .collect()
        };

        if align_to == "local" {
            // Align to local
            // Network move mapping
            let (pool, ctx, _output) = match build_pool_and_ctx(&local_cfg).await {
                Some(result) => result,
                None => return,
            };

            // Process mapping edit, errors are handled internally
            let _ = proc_mapping_edit(&pool, ctx, EditMappingActionArguments { operations }).await;
        } else if align_to == "remote" {
            // Align to remote
            // Offline move files
            for (remote_path, (_, local_path)) in operations {
                let local_path = local_path.unwrap();
                let from = local_dir.join(&local_path);
                let to = local_dir.join(&remote_path);

                if to.exists() {
                    eprintln!(
                        "{}",
                        md(t!(
                            "jv.fail.sheet.align.target_exists",
                            local = local_path.display(),
                            remote = remote_path.display()
                        ))
                    );
                    return;
                }

                if let Some(parent) = to.parent() {
                    if let Err(err) = fs::create_dir_all(parent).await {
                        eprintln!("{}", md(t!("jv.fail.sheet.align.move_failed", err = err)));
                        continue;
                    }
                } else {
                    eprintln!(
                        "{}",
                        md(t!(
                            "jv.fail.sheet.align.move_failed",
                            err = "no parent directory"
                        ))
                    );
                    continue;
                }
                if let Err(err) = fs::rename(from, to).await {
                    eprintln!("{}", md(t!("jv.fail.sheet.align.move_failed", err = err)));
                }
            }
        } else if align_to == "break" {
            for (remote_path, (_, _)) in operations {
                let Ok(mapping) = local_sheet.mapping_data_mut(&remote_path) else {
                    eprintln!(
                        "{}",
                        md(t!(
                            "jv.fail.sheet.align.mapping_not_found",
                            mapping = remote_path.display()
                        ))
                    );
                    return;
                };

                // Restore the latest detected hash to the original hash,
                // making the analyzer unable to correctly match
                //
                // That is to say,
                // if the file's hash has remained completely unchanged from the beginning to the end,
                // then break is also ineffective.
                mapping.set_last_modifiy_check_hash(Some(mapping.hash_when_updated().clone()));
            }

            // Save sheet
            let Ok(_) = local_sheet.write().await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim());
                return;
            };
        }
    }
    // Lost: match or confirm mode
    else if task.starts_with("lost") {
        let selected_lost_mapping: Vec<(AlignTaskName, PathBuf)> = align_tasks
            .lost
            .iter()
            .filter(|(name, _)| name.starts_with(&task))
            .cloned()
            .collect();

        if to == "confirm" {
            // Confirm mode
            for (_, path) in selected_lost_mapping {
                if let Err(err) = local_sheet.remove_mapping(&path) {
                    eprintln!(
                        "{}",
                        md(t!("jv.fail.sheet.align.remove_mapping_failed", err = err))
                    );
                };
            }
            // Save sheet
            let Ok(_) = local_sheet.write().await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim());
                return;
            };
            return;
        }

        if to.starts_with("created") {
            // Match mode
            let created_file: Vec<(AlignTaskName, PathBuf)> = align_tasks
                .created
                .iter()
                .find(|p| p.0.starts_with(&to))
                .map(|found| found.clone())
                .into_iter()
                .collect();

            if selected_lost_mapping.len() < 1 {
                eprintln!("{}", md(t!("jv.fail.sheet.align.no_lost_matched")));
                return;
            }

            if created_file.len() < 1 {
                eprintln!("{}", md(t!("jv.fail.sheet.align.no_created_matched")));
                return;
            }

            if selected_lost_mapping.len() > 1 {
                eprintln!("{}", md(t!("jv.fail.sheet.align.too_many_lost")));
                return;
            }

            if created_file.len() > 1 {
                eprintln!("{}", md(t!("jv.fail.sheet.align.too_many_created")));
                return;
            }

            // Check completed, match lost and created items
            let lost_mapping = &selected_lost_mapping.first().unwrap().1;
            let created_file = local_dir.join(&created_file.first().unwrap().1);

            let Ok(hash_calc) = sha1_hash::calc_sha1(&created_file, 4096usize).await else {
                eprintln!("{}", md(t!("jv.fail.sheet.align.calc_hash_failed")));
                return;
            };
            let Ok(mapping) = local_sheet.mapping_data_mut(lost_mapping) else {
                eprintln!(
                    "{}",
                    md(t!(
                        "jv.fail.sheet.align.mapping_not_found",
                        mapping = lost_mapping.display()
                    ))
                );
                return;
            };

            mapping.set_last_modifiy_check_hash(Some(hash_calc.hash));

            // Save sheet
            let Ok(_) = local_sheet.write().await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim());
                return;
            };
        }
    }
    // Erased: confirm mode
    else if task.starts_with("erased") {
        let selected_erased_mapping: Vec<(AlignTaskName, PathBuf)> = align_tasks
            .erased
            .iter()
            .filter(|(name, _)| name.starts_with(&task))
            .cloned()
            .collect();

        if to == "confirm" {
            // Confirm mode
            for (_, path) in selected_erased_mapping {
                if let Err(err) = local_sheet.remove_mapping(&path) {
                    eprintln!(
                        "{}",
                        md(t!("jv.fail.sheet.align.delete_mapping_failed", err = err))
                    );
                };

                let from = local_dir.join(&path);

                if !from.exists() {
                    continue;
                }

                let to = local_dir
                    .join(CLIENT_FOLDER_WORKSPACE_ROOT_NAME)
                    .join(".temp")
                    .join("erased")
                    .join(path);
                let to_path = to
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| to.clone());

                let _ = fs::create_dir_all(&to_path).await;
                if let Some(e) = fs::rename(&from, &to).await.err() {
                    eprintln!(
                        "{}",
                        md(t!(
                            "jv.fail.move.rename_failed",
                            from = from.display(),
                            to = to.display(),
                            error = e
                        ))
                        .yellow()
                    );
                }
            }

            // Save sheet
            let Ok(_) = local_sheet.write().await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim());
                return;
            };
            return;
        }
    }
}

async fn jv_track(args: TrackFileArgs) {
    // Perform glob operation before precheck, as precheck will call set_current_dir
    let track_files = if let Some(pattern) = args.track_file_pattern.clone() {
        let local_dir = match current_local_path() {
            Some(dir) => dir,
            None => {
                eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
                return;
            }
        };
        let files = glob(pattern, &local_dir).await;
        files
            .iter()
            .filter_map(|f| PathBuf::from_str(f.0).ok())
            .collect::<Vec<_>>()
    } else {
        println!("{}", md(t!("jv.track")));
        return;
    };

    // set_current_dir called here
    let local_config = match precheck().await {
        Some(config) => config,
        None => {
            return;
        }
    };

    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_config.clone()) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    if track_files.iter().len() < 1 {
        eprintln!("{}", md(t!("jv.fail.track.no_selection")));
        return;
    };

    let (pool, ctx, mut output) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    let files = track_files.iter().cloned().collect();
    let overwrite = args.allow_overwrite;
    let update_info = get_update_info(local_workspace, &files, args).await;

    let track_action = proc_track_file_action(
        &pool,
        ctx,
        TrackFileActionArguments {
            relative_pathes: files,
            file_update_info: update_info,
            print_infos: true,
            allow_overwrite_modified: overwrite,
        },
    );

    tokio::select! {
        result = track_action => {
            match result {
                Ok(result) => match result {
                    TrackFileActionResult::Done {
                        created,
                        updated,
                        synced,
                        skipped,
                    } => {
                        println!(
                            "{}",
                            md(t!(
                                "jv.result.track.done",
                                count = created.len() + updated.len() + synced.len(),
                                created = created.len(),
                                updated = updated.len(),
                                synced = synced.len()
                            ))
                        );

                        if skipped.len() > 0 {
                            println!(
                                "\n{}",
                                md(t!(
                                    "jv.result.track.tip_has_skipped",
                                    skipped_num = skipped.len(),
                                    skipped = skipped
                                        .iter()
                                        .map(|f| f.display().to_string())
                                        .collect::<Vec<String>>()
                                        .join("\n")
                                        .trim()
                                ))
                                .yellow()
                            );
                        }
                    }
                    TrackFileActionResult::AuthorizeFailed(e) => {
                        eprintln!("{}", md(t!("jv.result.common.authroize_failed", err = e)))
                    }
                    TrackFileActionResult::StructureChangesNotSolved => {
                        eprintln!("{}", md(t!("jv.result.track.structure_changes_not_solved")))
                    }
                    TrackFileActionResult::CreateTaskFailed(create_task_result) => match create_task_result
                    {
                        CreateTaskResult::Success(_) => {} // Success is not handled here
                        CreateTaskResult::CreateFileOnExistPath(path) => {
                            eprintln!(
                                "{}",
                                md(t!(
                                    "jv.result.track.create_failed.create_file_on_exist_path",
                                    path = path.display()
                                ))
                            )
                        }
                        CreateTaskResult::SheetNotFound(sheet) => {
                            eprintln!(
                                "{}",
                                md(t!(
                                    "jv.result.track.create_failed.sheet_not_found",
                                    name = sheet
                                ))
                            )
                        }
                    },
                    TrackFileActionResult::UpdateTaskFailed(update_task_result) => match update_task_result
                    {
                        UpdateTaskResult::Success(_) => {} // Success is not handled here
                        UpdateTaskResult::VerifyFailed { path, reason } => match reason {
                            VerifyFailReason::SheetNotFound(sheet_name) => {
                                eprintln!(
                                    "{}",
                                    md(t!(
                                        "jv.result.track.update_failed.verify.sheet_not_found",
                                        sheet_name = sheet_name
                                    ))
                                )
                            }
                            VerifyFailReason::MappingNotFound => {
                                eprintln!(
                                    "{}",
                                    md(t!(
                                        "jv.result.track.update_failed.verify.mapping_not_found",
                                        path = path.display()
                                    ))
                                )
                            }
                            VerifyFailReason::VirtualFileNotFound(vfid) => {
                                eprintln!(
                                    "{}",
                                    md(t!(
                                        "jv.result.track.update_failed.verify.virtual_file_not_found",
                                        vfid = vfid
                                    ))
                                )
                            }
                            VerifyFailReason::VirtualFileReadFailed(vfid) => {
                                eprintln!(
                                    "{}",
                                    md(t!(
                                        "jv.result.track.update_failed.verify.virtual_file_read_failed",
                                        vfid = vfid
                                    ))
                                )
                            }
                            VerifyFailReason::NotHeld => {
                                eprintln!(
                                    "{}",
                                    md(t!(
                                        "jv.result.track.update_failed.verify.not_held",
                                        path = path.display()
                                    ))
                                )
                            }
                            VerifyFailReason::VersionDismatch(current_version, latest_version) => {
                                eprintln!(
                                    "{}",
                                    md(t!(
                                        "jv.result.track.update_failed.verify.version_dismatch",
                                        version_current = current_version,
                                        version_latest = latest_version
                                    ))
                                )
                            }
                            VerifyFailReason::UpdateButNoDescription => {
                                eprintln!(
                                    "{}",
                                    md(t!(
                                        "jv.result.track.update_failed.verify.update_but_no_description"
                                    ))
                                )
                            }
                            VerifyFailReason::VersionAlreadyExist(latest_version) => {
                                eprintln!(
                                    "{}",
                                    md(t!(
                                        "jv.result.track.update_failed.verify.version_already_exist",
                                        path = path.display(),
                                        version = latest_version
                                    ))
                                )
                            }
                        },
                    },
                    TrackFileActionResult::SyncTaskFailed(sync_task_result) => match sync_task_result {
                        SyncTaskResult::Success(_) => {} // Success is not handled here
                    },
                },
                Err(e) => handle_err(e),
            }
        }
        _ = async {
            while let Some(msg) = output.recv().await {
                println!("{}", msg);
            }
        } => {}
    }
}

async fn get_update_info(
    workspace: LocalWorkspace,
    files: &HashSet<PathBuf>,
    args: TrackFileArgs,
) -> HashMap<PathBuf, (NextVersion, UpdateDescription)> {
    let mut result = HashMap::new();

    if files.len() == 1 {
        if let (Some(desc), Some(ver)) = (&args.desc, &args.next_version) {
            if let Some(file) = files.iter().next() {
                result.insert(file.clone(), (ver.clone(), desc.clone()));
                return result;
            }
        }
    }
    if args.work {
        return start_update_editor(workspace, files, &args).await;
    }

    result
}

async fn start_update_editor(
    workspace: LocalWorkspace,
    files: &HashSet<PathBuf>,
    args: &TrackFileArgs,
) -> HashMap<PathBuf, (NextVersion, UpdateDescription)> {
    let account = workspace.config().lock().await.current_account();

    let Ok(latest_file_data_path) = LatestFileData::data_path(&account) else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return HashMap::new();
    };

    // Get latest file data
    let Ok(latest_file_data) = LatestFileData::read_from(&latest_file_data_path).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return HashMap::new();
    };

    // Get files
    let Ok(analyzed) = AnalyzeResult::analyze_local_status(&workspace).await else {
        return HashMap::new();
    };
    // Has unsolved moves, skip
    if analyzed.lost.len() > 0 || analyzed.moved.len() > 0 {
        return HashMap::new();
    }
    // No modified, skip
    if analyzed.modified.len() < 1 {
        return HashMap::new();
    }
    // No sheet, skip
    let Some(sheet) = workspace.config().lock().await.sheet_in_use().clone() else {
        return HashMap::new();
    };
    // No cached sheet, skip
    let Ok(cached_sheet) = CachedSheet::cached_sheet_data(&sheet).await else {
        return HashMap::new();
    };
    let files: Vec<(PathBuf, VirtualFileVersion)> = files
        .iter()
        .filter_map(|file| {
            if analyzed.modified.contains(file) {
                if let Some(mapping_item) = cached_sheet.mapping().get(file) {
                    if let Some(latest_version) = latest_file_data.file_version(&mapping_item.id) {
                        return Some((file.clone(), latest_version.clone()));
                    }
                }
                None
            } else {
                None
            }
        })
        .collect();

    // Generate editor text
    let mut table = SimpleTable::new_with_padding(
        vec![
            t!("editor.modified_line.header.file_path").trim(),
            t!("editor.modified_line.header.old_version").trim(),
            "",
            t!("editor.modified_line.header.new_version").trim(),
        ],
        2,
    );
    for item in files {
        let path = item.0.display().to_string();
        let base_ver = item.1.to_string();
        let next_ver = push_version(&base_ver).unwrap_or(" ".to_string());
        table.push_item(vec![
            path,
            base_ver,
            t!("editor.modified_line.content.arrow").trim().to_string(),
            next_ver,
        ]);
    }
    let lines = table.to_string();

    let str = t!(
        "editor.update_editor",
        modified_lines = lines,
        description = args.desc.clone().unwrap_or_default()
    );

    let path = workspace
        .local_path()
        .join(CLIENT_PATH_WORKSPACE_ROOT)
        .join(".UPDATE.md");
    let result = input_with_editor(str, path, "#").await.unwrap_or_default();

    let mut update_info = HashMap::new();

    // Parse the result returned from the editor
    let lines: Vec<&str> = result.lines().collect();
    let mut i = 0;

    // Find the separator line
    let mut separator_index = None;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.chars().all(|c| c == '-') && line.len() >= 5 {
            separator_index = Some(i);
            break;
        }
        i += 1;
    }

    if let Some(sep_idx) = separator_index {
        // Parse path and version information before the separator
        for line in &lines[..sep_idx] {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue;
            }

            // Parse format: /directory/file.extension version -> new_version
            if let Some(arrow_pos) = trimmed_line.find("->") {
                let before_arrow = &trimmed_line[..arrow_pos].trim();
                let after_arrow = &trimmed_line[arrow_pos + 2..].trim();

                // Separate path and old version
                if let Some(last_space) = before_arrow.rfind(' ') {
                    let path_str = &before_arrow[..last_space].trim();
                    let _old_version = &before_arrow[last_space + 1..].trim(); // Old version, needs parsing but not used
                    let new_version = after_arrow.trim();

                    if !path_str.is_empty() && !new_version.is_empty() {
                        let path = PathBuf::from(path_str);
                        // Get description (all content after the separator)
                        let description = lines[sep_idx + 1..].join("\n").trim().to_string();

                        // Only add to update_info if description is not empty after trimming
                        if !description.is_empty() {
                            update_info.insert(path, (new_version.to_string(), description));
                        }
                    }
                }
            }
        }
    }

    update_info
}

async fn jv_hold(args: HoldFileArgs) {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Some(hold_file_pattern) = args.hold_file_pattern else {
        println!("{}", md(t!("jv.hold")));
        return;
    };

    let files = glob(hold_file_pattern, &local_dir).await;

    let _ = correct_current_dir();

    jv_change_edit_right(
        files
            .iter()
            .filter_map(|f| PathBuf::from_str(f.0).ok())
            .collect(),
        EditRightChangeBehaviour::Hold,
        args.show_fail_details,
        args.skip_failed,
        args.force,
    )
    .await;
}

async fn jv_throw(args: ThrowFileArgs) {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Some(throw_file_pattern) = args.throw_file_pattern else {
        println!("{}", md(t!("jv.throw")));
        return;
    };

    let files = glob(throw_file_pattern, &local_dir).await;

    let _ = correct_current_dir();

    jv_change_edit_right(
        files
            .iter()
            .filter_map(|f| PathBuf::from_str(f.0).ok())
            .collect(),
        EditRightChangeBehaviour::Throw,
        args.show_fail_details,
        args.skip_failed,
        args.force,
    )
    .await;
}

async fn jv_change_edit_right(
    files: Vec<PathBuf>,
    behaviour: EditRightChangeBehaviour,
    show_fail_details: bool,
    mut skip_failed: bool,
    force: bool,
) {
    // If both `--details` and `--skip-failed` are set, only enable `--details`
    if show_fail_details && skip_failed {
        skip_failed = false;
    }

    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Ok(local_cfg) = LocalConfig::read_from(local_dir.join(CLIENT_FILE_WORKSPACE)).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_cfg.clone()) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    // Get files
    let Ok(analyzed) = AnalyzeResult::analyze_local_status(&local_workspace).await else {
        eprintln!("{}", md(t!("jv.fail.status.analyze")).trim());
        return;
    };

    let account = local_cfg.current_account();

    let Ok(latest_file_data_path) = LatestFileData::data_path(&account) else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(latest_file_data) = LatestFileData::read_from(&latest_file_data_path).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Some(sheet_name) = local_cfg.sheet_in_use().clone() else {
        eprintln!("{}", md(t!("jv.fail.status.no_sheet_in_use")).trim());
        return;
    };

    let Ok(local_sheet) = local_workspace.local_sheet(&account, &sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(cached_sheet) = CachedSheet::cached_sheet_data(&sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let num = files.iter().len();
    if num < 1 {
        eprintln!("{}", md(t!("jv.fail.change_edit_right.no_selection")));
        return;
    }

    let mut passed_files = Vec::new();
    let mut details = Vec::new();
    let mut failed = 0;

    // Helper function to handle validation failures
    fn handle_validation_failure(
        show_fail_details: bool,
        details: &mut Vec<String>,
        failed: &mut usize,
        num: usize,
        reason: String,
    ) -> bool {
        if show_fail_details {
            details.push(reason);
            *failed += 1;
            true // Continue to next file
        } else {
            eprintln!(
                "{}",
                md(t!("jv.fail.change_edit_right.check_failed", num = num))
            );
            false // Break processing
        }
    }

    for file in files {
        let exists = file.exists();

        // If force is enabled, add to the list regardless
        if force {
            passed_files.push(file);
            continue;
        }

        // Mapping exists
        let Some(cached_mapping) = cached_sheet.mapping().get(&file) else {
            let reason = t!(
                "jv.fail.change_edit_right.check_fail_item",
                path = file.display(),
                reason = t!("jv.fail.change_edit_right.check_fail_reason.not_found_in_sheet")
            )
            .trim()
            .to_string();

            if !handle_validation_failure(show_fail_details, &mut details, &mut failed, num, reason)
            {
                return;
            }
            continue;
        };

        let vfid: VirtualFileId = if exists {
            // Not tracked
            let Ok(local_mapping) = local_sheet.mapping_data(&file) else {
                let reason = t!(
                    "jv.fail.change_edit_right.check_fail_item",
                    path = file.display(),
                    reason = t!("jv.fail.change_edit_right.check_fail_reason.not_a_tracked_file")
                )
                .trim()
                .to_string();

                if !handle_validation_failure(
                    show_fail_details,
                    &mut details,
                    &mut failed,
                    num,
                    reason,
                ) {
                    return;
                }
                continue;
            };

            let vfid = local_mapping.mapping_vfid();
            let local_version = local_mapping.version_when_updated();

            // Base version unmatch
            if local_version
                != latest_file_data
                    .file_version(vfid)
                    .unwrap_or(&String::default())
            {
                let reason = t!(
                    "jv.fail.change_edit_right.check_fail_item",
                    path = file.display(),
                    reason = t!("jv.fail.change_edit_right.check_fail_reason.base_version_unmatch")
                )
                .trim()
                .to_string();

                if !handle_validation_failure(
                    show_fail_details,
                    &mut details,
                    &mut failed,
                    num,
                    reason,
                ) {
                    return;
                }
                continue;
            }

            vfid.clone()
        } else {
            cached_mapping.id.clone()
        };

        // Hold validation
        let holder = latest_file_data.file_holder(&vfid);
        let validation_passed = match behaviour {
            EditRightChangeBehaviour::Hold => {
                if holder.is_some_and(|h| h != &account) {
                    // Has holder but not current account
                    let holder_name = holder.unwrap();
                    let reason = t!(
                        "jv.fail.change_edit_right.check_fail_item",
                        path = file.display(),
                        reason = t!(
                            "jv.fail.change_edit_right.check_fail_reason.has_holder",
                            holder = holder_name
                        )
                    )
                    .trim()
                    .to_string();

                    if !handle_validation_failure(
                        show_fail_details,
                        &mut details,
                        &mut failed,
                        num,
                        reason,
                    ) {
                        return;
                    }
                    false
                } else if holder.is_some_and(|h| h == &account) {
                    // Already held by current account
                    let reason = t!(
                        "jv.fail.change_edit_right.check_fail_item",
                        path = file.display(),
                        reason = t!("jv.fail.change_edit_right.check_fail_reason.already_held")
                    )
                    .trim()
                    .to_string();

                    if !handle_validation_failure(
                        show_fail_details,
                        &mut details,
                        &mut failed,
                        num,
                        reason,
                    ) {
                        return;
                    }
                    false
                } else {
                    true
                }
            }
            EditRightChangeBehaviour::Throw => {
                if holder.is_some_and(|h| h != &account) {
                    // Not the holder
                    let reason = t!(
                        "jv.fail.change_edit_right.check_fail_item",
                        path = file.display(),
                        reason = t!("jv.fail.change_edit_right.check_fail_reason.not_holder")
                    )
                    .trim()
                    .to_string();

                    if !handle_validation_failure(
                        show_fail_details,
                        &mut details,
                        &mut failed,
                        num,
                        reason,
                    ) {
                        return;
                    }
                    false
                } else if analyzed.modified.contains(&file) {
                    // Already modified
                    let reason = t!(
                        "jv.fail.change_edit_right.check_fail_item",
                        path = file.display(),
                        reason = t!("jv.fail.change_edit_right.check_fail_reason.already_modified")
                    )
                    .trim()
                    .to_string();

                    if !handle_validation_failure(
                        show_fail_details,
                        &mut details,
                        &mut failed,
                        num,
                        reason,
                    ) {
                        return;
                    }
                    false
                } else {
                    true
                }
            }
        };

        if validation_passed {
            passed_files.push(file);
        }
    }

    if failed > 0 && show_fail_details {
        eprintln!(
            "{}",
            md(t!(
                "jv.fail.change_edit_right.check_failed_details",
                num = num,
                failed = failed,
                items = details.join("\n").trim().yellow()
            ))
        );
        return;
    }

    if !(failed > 0 && skip_failed) && failed != 0 {
        return;
    }

    let (pool, ctx, _output) = match build_pool_and_ctx(&local_cfg).await {
        Some(result) => result,
        None => return,
    };

    let passed = passed_files
        .iter()
        .map(|f| (f.clone(), behaviour.clone()))
        .collect();

    match proc_change_virtual_file_edit_right_action(&pool, ctx, (passed, true)).await {
        Ok(r) => match r {
            ChangeVirtualFileEditRightResult::Success {
                success_hold,
                success_throw,
            } => {
                if success_hold.len() > 0 && success_throw.len() == 0 {
                    println!(
                        "{}",
                        md(t!(
                            "jv.result.change_edit_right.success.hold",
                            num = success_hold.len()
                        ))
                    )
                } else if success_hold.len() == 0 && success_throw.len() > 0 {
                    println!(
                        "{}",
                        md(t!(
                            "jv.result.change_edit_right.success.throw",
                            num = success_throw.len()
                        ))
                    )
                } else if success_hold.len() > 0 && success_throw.len() > 0 {
                    println!(
                        "{}",
                        md(t!(
                            "jv.result.change_edit_right.success.mixed",
                            num = success_hold.len() + success_throw.len(),
                            num_hold = success_hold.len(),
                            num_throw = success_throw.len()
                        ))
                    )
                } else {
                    eprintln!("{}", md(t!("jv.result.change_edit_right.failed.none")))
                }
            }
            ChangeVirtualFileEditRightResult::AuthorizeFailed(e) => {
                eprintln!("{}", md(t!("jv.result.common.authroize_failed", err = e)))
            }
            ChangeVirtualFileEditRightResult::DoNothing => {
                eprintln!("{}", md(t!("jv.result.change_edit_right.failed.none")))
            }
        },
        Err(e) => handle_err(e),
    }
}

async fn jv_move(args: MoveMappingArgs) {
    let local_dir = match current_local_path() {
        Some(dir) => dir,
        None => {
            eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
            return;
        }
    };

    let move_files = if let Some(from_pattern) = args.move_mapping_pattern.clone() {
        let from = glob(from_pattern, &local_dir).await;
        from.iter()
            .filter_map(|f| PathBuf::from_str(f.0).ok())
            .collect::<Vec<_>>()
    } else {
        println!("{}", md(t!("jv.move")));
        return;
    };

    let to_pattern = if args.to_mapping_pattern.is_some() {
        args.to_mapping_pattern.unwrap()
    } else {
        if args.erase {
            "".to_string()
        } else {
            eprintln!("{}", md(t!("jv.fail.move.no_target_dir")));
            return;
        }
    };

    let is_to_pattern_a_dir = to_pattern.ends_with('/') || to_pattern.ends_with('\\');

    let from_mappings = move_files
        .iter()
        .map(|f| f.display().to_string())
        .collect::<Vec<_>>();

    let base_path = Globber::from(&to_pattern).base().clone();
    let base_path = format_path(base_path.strip_prefix(&local_dir).unwrap().join("./")).unwrap();
    let to_path = base_path.join(to_pattern);

    let mut edit_mapping_args: EditMappingActionArguments = EditMappingActionArguments {
        operations: HashMap::<FromRelativePathBuf, OperationArgument>::new(),
    };

    if args.erase {
        // Generate erase operation parameters
        for from_mapping in from_mappings {
            edit_mapping_args
                .operations
                .insert(from_mapping.into(), (EditMappingOperations::Erase, None));
        }
    } else {
        // Generate move operation parameters
        // Single file move
        if from_mappings.len() == 1 {
            let from = format_path_str(from_mappings[0].clone()).unwrap();
            let to = if is_to_pattern_a_dir {
                // Input is a directory, append the filename
                format_path(
                    to_path
                        .join(from.strip_prefix(&base_path.display().to_string()).unwrap())
                        .to_path_buf(),
                )
                .unwrap()
            } else {
                // Input is a filename, use it directly
                format_path(to_path.to_path_buf()).unwrap()
            };

            let from: PathBuf = from.into();
            // If the from path contains to_path, ignore it to avoid duplicate moves
            if !from.starts_with(to_path) {
                edit_mapping_args
                    .operations
                    .insert(from, (EditMappingOperations::Move, Some(to.clone())));
            }
        } else
        // Multiple file move
        if from_mappings.len() > 1 && is_to_pattern_a_dir {
            let to_path = format_path(to_path).unwrap();
            for p in &from_mappings {
                let name = p.strip_prefix(&base_path.display().to_string()).unwrap();
                let to = format_path(to_path.join(name))
                    .unwrap()
                    .display()
                    .to_string();

                let from: PathBuf = p.into();
                // If the from path contains to_path, ignore it to avoid duplicate moves
                if !from.starts_with(to_path.display().to_string()) {
                    edit_mapping_args
                        .operations
                        .insert(from, (EditMappingOperations::Move, Some(to.into())));
                }
            }
        }
        if from_mappings.len() > 1 && !is_to_pattern_a_dir {
            eprintln!("{}", md(t!("jv.fail.move.count_doesnt_match")));
            return;
        }

        // NOTE
        // if move_file_mappings.len() < 1 {
        //      This case has already been handled earlier: output Help
        // }
    }

    let local_cfg = match precheck().await {
        Some(config) => config,
        None => return,
    };

    let (pool, ctx, _output) = match build_pool_and_ctx(&local_cfg).await {
        Some(result) => result,
        None => return,
    };

    if proc_mapping_edit(&pool, ctx, edit_mapping_args.clone())
        .await
        .is_ok()
    {
        // If the operation succeeds and only_remote is not enabled,
        // synchronize local moves
        if !args.only_remote {
            let erase_dir = local_dir
                .join(CLIENT_FOLDER_WORKSPACE_ROOT_NAME)
                .join(".temp")
                .join("erased");

            let mut skipped = 0;
            for (from_relative, (operation, to_relative)) in edit_mapping_args.operations {
                let from = local_dir.join(&from_relative);

                if !from.exists() {
                    continue;
                }

                let to = match operation {
                    EditMappingOperations::Move => local_dir.join(to_relative.unwrap()),
                    EditMappingOperations::Erase => erase_dir.join(&from_relative),
                };
                if let Some(to_dir) = to.parent() {
                    let _ = fs::create_dir_all(to_dir).await;
                }
                if let Some(e) = fs::rename(&from, &to).await.err() {
                    eprintln!(
                        "{}",
                        md(t!(
                            "jv.fail.move.rename_failed",
                            from = from.display(),
                            to = to.display(),
                            error = e
                        ))
                        .yellow()
                    );
                    skipped += 1;
                }
            }
            if skipped > 0 {
                eprintln!("{}", md(t!("jv.fail.move.has_rename_failed")));
            }
        }
    }
}

async fn proc_mapping_edit(
    pool: &ActionPool,
    ctx: ActionContext,
    edit_mapping_args: EditMappingActionArguments,
) -> Result<(), ()> {
    match proc_edit_mapping_action(
        pool,
        ctx,
        EditMappingActionArguments {
            operations: edit_mapping_args.operations.clone(),
        },
    )
    .await
    {
        Ok(r) => match r {
            EditMappingActionResult::Success => {
                println!("{}", md(t!("jv.result.move.success")));
                Ok(())
            }
            EditMappingActionResult::AuthorizeFailed(e) => {
                eprintln!("{}", md(t!("jv.result.common.authroize_failed", err = e)));
                Err(())
            }
            EditMappingActionResult::MappingNotFound(path_buf) => {
                eprintln!(
                    "{}",
                    md(t!(
                        "jv.result.move.mapping_not_found",
                        path = path_buf.display()
                    ))
                );
                Err(())
            }
            EditMappingActionResult::InvalidMove(invalid_move_reason) => {
                match invalid_move_reason {
                    InvalidMoveReason::MoveOperationButNoTarget(path_buf) => {
                        eprintln!(
                            "{}",
                            md(t!(
                                "jv.result.move.invalid_move.no_target",
                                path = path_buf.display()
                            ))
                        );
                    }
                    InvalidMoveReason::ContainsDuplicateMapping(path_buf) => {
                        eprintln!(
                            "{}",
                            md(t!(
                                "jv.result.move.invalid_move.duplicate_mapping",
                                path = path_buf.display()
                            ))
                        );
                    }
                }
                Err(())
            }
            EditMappingActionResult::Unknown => {
                eprintln!("{}", md(t!("jv.result.move.unknown")));
                Err(())
            }
            EditMappingActionResult::EditNotAllowed => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.common.not_allowed_in_reference_sheet"))
                );
                Err(())
            }
        },
        Err(e) => {
            handle_err(e);
            Err(())
        }
    }
}

async fn jv_share(args: ShareFileArgs) {
    // Import share mode
    if let (Some(import_id), None, None) = (&args.args1, &args.args2, &args.args3) {
        share_accept(import_id).await;
        return;
    }

    // Pull mode
    if let (Some(from_sheet), Some(import_pattern), None) = (&args.args1, &args.args2, &args.args3)
    {
        share_in(from_sheet, import_pattern).await;
        return;
    }

    // Share mode
    if let (Some(share_pattern), Some(to_sheet), Some(description)) =
        (&args.args1, &args.args2, &args.args3)
    {
        share_out(share_pattern, to_sheet, description).await;
        return;
    }

    println!("{}", md(t!("jv.share")));
}

async fn share_accept(_import_id: &str) {
    // TODO: Implement import share logic
    eprintln!("share_accept not implemented yet");
}

async fn share_in(_from_sheet: &str, _import_pattern: &str) {
    // TODO: Implement pull mode logic
    eprintln!("share_in not implemented yet");
}

async fn share_out(_share_pattern: &str, _to_sheet: &str, _description: &str) {
    // TODO: Implement share mode logic
    eprintln!("share_out not implemented yet");
}

async fn jv_account_add(user_dir: UserDirectory, args: AccountAddArgs) {
    let member = Member::new(args.account_name.clone());

    match user_dir.register_account(member).await {
        Ok(_) => {
            println!(
                "{}",
                t!("jv.success.account.add", account = args.account_name)
            );
        }
        Err(_) => {
            eprintln!("{}", t!("jv.fail.account.add", account = args.account_name));
            return;
        }
    }
    if args.keygen {
        let output_path = current_dir().unwrap().join("tempkey.pem");

        match Command::new("openssl")
            .args([
                "genpkey",
                "-algorithm",
                "ed25519",
                "-out",
                &output_path.to_string_lossy(),
            ])
            .status()
            .await
        {
            Ok(status) if status.success() => {
                jv_account_move_key(
                    user_dir,
                    MoveKeyToAccountArgs {
                        help: false,
                        account_name: args.account_name,
                        key_path: output_path,
                    },
                )
                .await
            }
            Ok(_) => {
                eprintln!("{}", t!("jv.fail.account.keygen"));
            }
            Err(_) => {
                eprintln!("{}", t!("jv.fail.account.keygen_exec"));
            }
        }
    }
}

async fn jv_account_remove(user_dir: UserDirectory, args: AccountRemoveArgs) {
    match user_dir.remove_account(&args.account_name) {
        Ok(_) => {
            println!(
                "{}",
                t!("jv.success.account.remove", account = args.account_name)
            );
        }
        Err(_) => {
            eprintln!(
                "{}",
                t!("jv.fail.account.remove", account = args.account_name)
            );
        }
    }
}

async fn jv_account_list(user_dir: UserDirectory, args: AccountListArgs) {
    let _ = correct_current_dir();

    if args.raw {
        let Ok(account_ids) = user_dir.account_ids() else {
            return;
        };
        account_ids.iter().for_each(|a| println!("host/{}", a));
        account_ids.iter().for_each(|a| println!("{}", a));
        return;
    }

    match user_dir.account_ids() {
        Ok(account_ids) => {
            println!(
                "{}",
                md(t!(
                    "jv.success.account.list.header",
                    num = account_ids.len()
                ))
            );

            let mut i = 0;
            for account_id in account_ids {
                println!("{}. {} {}", i + 1, &account_id, {
                    if user_dir.has_private_key(&account_id) {
                        t!("jv.success.account.list.status_has_key")
                    } else {
                        std::borrow::Cow::Borrowed("")
                    }
                });
                i += 1;
            }
        }
        Err(_) => {
            eprintln!("{}", t!("jv.fail.account.list"));
        }
    }
}

async fn jv_account_as(user_dir: UserDirectory, args: SetLocalWorkspaceAccountArgs) {
    let (account, is_host_mode) = process_account_parameter(args.account_name);

    // Account exist
    let Ok(member) = user_dir.account(&account).await else {
        eprintln!("{}", t!("jv.fail.account.not_found", account = &account));
        return;
    };

    let Some(_local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    if let Err(_) = local_cfg.set_current_account(member.id()) {
        eprintln!("{}", md(t!("jv.fail.account.as")));
        return;
    };

    local_cfg.set_host_mode(is_host_mode);

    let Ok(_) = LocalConfig::write(&local_cfg).await else {
        eprintln!("{}", t!("jv.fail.write_cfg").trim());
        return;
    };

    if is_host_mode {
        println!(
            "{}",
            md(t!("jv.success.account.as_host", account = member.id()))
        );
    } else {
        println!(
            "{}",
            t!("jv.success.account.as", account = member.id()).trim()
        );
    }
}

/// Input account, get MemberId and whether it's a host
/// If input is host/xxx, output is xxx, true
/// If input is xxx, output is xxx, false
fn process_account_parameter(account_input: String) -> (MemberId, bool) {
    let is_host = account_input.starts_with("host/");
    let account_id = if is_host {
        account_input.strip_prefix("host/").unwrap().to_string()
    } else {
        account_input
    };
    (snake_case!(account_id), is_host)
}

async fn jv_account_move_key(user_dir: UserDirectory, args: MoveKeyToAccountArgs) {
    // Key file exist
    if !args.key_path.exists() {
        eprintln!(
            "{}",
            t!("jv.fail.path_not_found", path = args.key_path.display())
        );
        return;
    }

    // Account exist
    let Ok(_member) = user_dir.account(&args.account_name).await else {
        eprintln!(
            "{}",
            t!("jv.fail.account.not_found", account = args.account_name)
        );
        return;
    };

    // Rename key file
    match move_across_partitions(
        args.key_path,
        user_dir.account_private_key_path(&args.account_name),
    )
    .await
    {
        Ok(_) => println!("{}", t!("jv.success.account.move_key")),
        Err(_) => eprintln!("{}", t!("jv.fail.account.move_key")),
    }
}

async fn jv_account_generate_pub_key(user_dir: UserDirectory, args: GeneratePublicKeyArgs) {
    let private_key_path = user_dir.account_private_key_path(&args.account_name);
    let target_path = args
        .output_dir
        .unwrap_or(current_dir().unwrap())
        .join(format!("{}.pem", args.account_name));

    match Command::new("openssl")
        .args([
            "pkey",
            "-in",
            &private_key_path.to_string_lossy(),
            "-pubout",
            "-out",
            &target_path.to_string_lossy(),
        ])
        .status()
        .await
    {
        Ok(status) if status.success() => {
            println!(
                "{}",
                t!(
                    "jv.success.account.generate_pub_key",
                    export = target_path.display()
                )
            );
        }
        Ok(_) => {
            eprintln!("{}", t!("jv.fail.account.keygen"));
        }
        Err(_) => {
            eprintln!("{}", t!("jv.fail.account.keygen_exec"));
        }
    }
}

async fn jv_update(update_file_args: UpdateArgs) {
    let local_config = match precheck().await {
        Some(config) => config,
        None => return,
    };

    let (pool, ctx, _output) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    match proc_update_to_latest_info_action(&pool, ctx, ()).await {
        Err(e) => handle_err(e),
        Ok(result) => {
            if !update_file_args.silent {
                match result {
                    UpdateToLatestInfoResult::Success => {
                        println!("{}", md(t!("jv.result.update.success")));
                    }
                    UpdateToLatestInfoResult::AuthorizeFailed(e) => {
                        eprintln!("{}", md(t!("jv.result.common.authroize_failed", err = e)))
                    }
                    UpdateToLatestInfoResult::SyncCachedSheetFail(
                        sync_cached_sheet_fail_reason,
                    ) => match sync_cached_sheet_fail_reason {
                        SyncCachedSheetFailReason::PathAlreadyExist(path_buf) => {
                            eprintln!(
                                "{}",
                                md(t!(
                                    "jv.result.update.fail.sync_cached_sheet_fail.path_already_exist",
                                    path = path_buf.display()
                                ))
                            );
                        }
                    },
                }
            }
        }
    }
}

async fn jv_direct(args: DirectArgs) {
    let Some(upstream) = args.upstream else {
        println!("{}", md(t!("jv.direct")));
        return;
    };

    if !args.confirm {
        println!(
            "{}",
            t!("jv.confirm.direct", upstream = upstream).trim().yellow()
        );
        confirm_hint_or(t!("common.confirm"), || exit(1)).await;
    }

    let pool = client_registry::client_action_pool();
    let upstream = match socket_addr_helper::get_socket_addr(&upstream, PORT).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!(
                "{}",
                md(t!(
                    "jv.fail.parse.str_to_sockaddr",
                    str = &upstream.trim(),
                    err = e
                ))
            );
            return;
        }
    };

    let Some(instance) = connect(upstream).await else {
        // Since connect() function already printed error messages, we only handle the return here
        return;
    };

    let ctx = ActionContext::local().insert_instance(instance);

    match proc_set_upstream_vault_action(&pool, ctx, upstream).await {
        Err(e) => handle_err(e),
        Ok(result) => match result {
            SetUpstreamVaultActionResult::DirectedAndStained => {
                println!(
                    "{}",
                    md(t!(
                        "jv.result.direct.directed_and_stained",
                        upstream = upstream
                    ))
                );
                insert_recent_ip_address(upstream.to_string().trim()).await;
            }
            SetUpstreamVaultActionResult::Redirected => {
                println!(
                    "{}",
                    md(t!("jv.result.direct.redirected", upstream = upstream))
                );
                insert_recent_ip_address(upstream.to_string().trim()).await;
            }
            SetUpstreamVaultActionResult::AlreadyStained => {
                eprintln!("{}", md(t!("jv.result.direct.already_stained")))
            }
            SetUpstreamVaultActionResult::AuthorizeFailed(e) => {
                eprintln!("{}", md(t!("jv.result.common.authroize_failed", err = e)))
            }
            SetUpstreamVaultActionResult::RedirectFailed(e) => {
                eprintln!("{}", md(t!("jv.result.direct.redirect_failed", err = e)))
            }
            SetUpstreamVaultActionResult::SameUpstream => {
                eprintln!("{}", md(t!("jv.result.direct.same_upstream")))
            }
            _ => {}
        },
    };
}

async fn jv_unstain(args: UnstainArgs) {
    let Some(_local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    if !local_cfg.stained() {
        eprintln!("{}", md(t!("jv.fail.unstain")));
        return;
    }

    if !args.confirm {
        println!(
            "{}",
            md(t!("jv.confirm.unstain", upstream = local_cfg.vault_addr())).yellow()
        );
        confirm_hint_or(t!("common.confirm"), || exit(1)).await;
    }

    local_cfg.unstain();

    let Ok(_) = LocalConfig::write(&local_cfg).await else {
        eprintln!("{}", t!("jv.fail.write_cfg").trim());
        return;
    };

    println!("{}", md(t!("jv.success.unstain")));
}

async fn jv_docs(args: DocsArgs) {
    let Some(docs_name) = args.docs_name else {
        if !args.raw {
            println!("{}", md(t!("jv.docs")));
        }
        return;
    };

    if docs_name.trim() == "ls" || docs_name.trim() == "list" {
        let docs = documents();

        if args.raw {
            docs.iter().for_each(|d| {
                if d.starts_with("docs_") {
                    println!("{}", d.trim_start_matches("docs_"))
                }
            });
            return;
        }

        println!("{}", md(t!("jv.success.docs.list.header")));
        let mut i = 0;
        for doc in docs {
            if doc.starts_with("docs_") {
                println!(
                    "{}",
                    md(t!(
                        "jv.success.docs.list.item",
                        num = i + 1,
                        docs_name = doc.trim_start_matches("docs_")
                    ))
                );
                i += 1;
            }
        }
        println!("{}", md(t!("jv.success.docs.list.footer")));

        return;
    }

    let name = format!("docs_{}", snake_case!(docs_name.clone()));
    let Some(document) = document(name) else {
        eprintln!(
            "{}",
            md(t!("jv.fail.docs.not_found", docs_name = docs_name))
        );
        return;
    };

    if args.direct {
        println!("{}", document.trim());
    } else {
        let Some(doc_dir) = current_cfg_dir() else {
            eprintln!(
                "{}",
                md(t!("jv.fail.docs.no_doc_dir", docs_name = docs_name))
            );
            return;
        };
        let file = doc_dir.join("DOCS.MD");
        if let Err(e) = show_in_pager(document, file).await {
            eprintln!(
                "{}",
                md(t!(
                    "jv.fail.docs.open_editor",
                    err = e,
                    docs_name = docs_name
                ))
            );
        }
    }
}

async fn jv_debug_glob(glob_args: DebugGlobArgs) {
    let local_dir = match current_local_path() {
        Some(dir) => dir,
        None => {
            // No, dont print anything
            // eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
            return;
        }
    };

    for path in glob(glob_args.pattern, &local_dir).await.keys() {
        println!("{}", path);
    }
}

async fn glob(pattern: impl Into<String>, local_dir: &PathBuf) -> BTreeMap<String, ()> {
    let pattern = pattern.into();
    let globber = match get_globber(&pattern, true).await {
        Ok(g) => g,
        Err(_) => match get_globber(&pattern, false).await {
            Ok(g) => g,
            Err(_) => return BTreeMap::new(),
        },
    };

    let result = globber.names();
    let base_dir = globber.base();

    let relative_path = base_dir
        .strip_prefix(local_dir)
        .unwrap_or(local_dir.as_path());

    let mut filtered_paths: Vec<String> = result
        .into_iter()
        .filter_map(|name| Some(relative_path.join(name).display().to_string()))
        .collect();

    let path_map: BTreeMap<String, ()> = filtered_paths.drain(..).map(|path| (path, ())).collect();
    path_map
}

async fn get_globber(
    pattern: impl Into<String>,
    with_current_sheet: bool,
) -> Result<Globber, std::io::Error> {
    // Build globber
    let globber = Globber::from(pattern.into());

    let globber = if with_current_sheet {
        // Get necessary informations
        let Some(local_dir) = current_local_path() else {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Workspace not found",
            ));
        };

        let Ok(local_cfg) = LocalConfig::read_from(local_dir.join(CLIENT_FILE_WORKSPACE)).await
        else {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Local Config read failed",
            ));
        };
        let Some(sheet_name) = local_cfg.sheet_in_use().clone() else {
            return Err(Error::new(std::io::ErrorKind::NotFound, "No sheet in use"));
        };

        let Ok(cached_sheet) = CachedSheet::cached_sheet_data(&sheet_name).await else {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Cached sheet not found",
            ));
        };

        let current_dir = current_dir()?;

        if !current_dir.starts_with(&local_dir) {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Not a local workspace",
            ));
        }

        // Sheet mode
        globber.glob(|current_dir| {
            let mut result = HashSet::new();

            // First, add local files
            get_local_files(&current_dir, &mut result);

            // Start collecting sheet files
            // Check if we're in the workspace directory (get current path relative to local workspace)
            let Ok(relative_path_to_local) = current_dir.strip_prefix(&local_dir) else {
                return result.into_iter().collect();
            };

            let mut dirs = HashSet::new();

            cached_sheet.mapping().iter().for_each(|(path, _)| {
                let left = relative_path_to_local;

                // Skip: files that don't start with the current directory
                let Ok(right) = path.strip_prefix(left) else {
                    return;
                };

                // File: starts with current directory and doesn't contain "/"
                // (since we already filtered out files that don't start with current directory,
                // here we just check if it contains a path separator)
                let file_name = right.display().to_string();
                if !file_name.contains("/") {
                    result.insert(GlobItem::File(file_name));
                } else {
                    // Directory: contains separator, take the first part, add to dirs set
                    if let Some(first_part) = file_name.split('/').next() {
                        dirs.insert(first_part.to_string());
                    }
                }
            });

            dirs.into_iter().for_each(|dir| {
                result.insert(GlobItem::Directory(dir));
            });

            result.into_iter().collect()
        })
    } else {
        // Local mode
        globber.glob(|current| {
            let mut items = HashSet::new();
            get_local_files(&current, &mut items);
            items.iter().cloned().collect()
        })
    }?;

    Ok(globber)
}

fn get_local_files(current: &PathBuf, items: &mut HashSet<GlobItem>) {
    if let Ok(entries) = std::fs::read_dir(&current) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

            if path.is_file() {
                items.insert(GlobItem::File(name));
            } else if path.is_dir() {
                if name != CLIENT_FOLDER_WORKSPACE_ROOT_NAME {
                    items.insert(GlobItem::Directory(name));
                }
            }
        }
    }
}

pub fn handle_err(err: TcpTargetError) {
    eprintln!("{}", md(t!("jv.fail.from_core", err = err)))
}

async fn connect(upstream: SocketAddr) -> Option<ConnectionInstance> {
    // Create Socket
    let socket = if upstream.is_ipv4() {
        match TcpSocket::new_v4() {
            Ok(socket) => socket,
            Err(_) => {
                eprintln!("{}", t!("jv.fail.create_socket").trim());
                return None;
            }
        }
    } else {
        match TcpSocket::new_v6() {
            Ok(socket) => socket,
            Err(_) => {
                eprintln!("{}", t!("jv.fail.create_socket").trim());
                return None;
            }
        }
    };

    // Connect
    let Ok(stream) = socket.connect(upstream).await else {
        eprintln!("{}", t!("jv.fail.connection_failed").trim());
        return None;
    };

    Some(ConnectionInstance::from(stream))
}

// Check if the workspace is stained and has a valid configuration
// Returns LocalConfig if valid, None otherwise
async fn precheck() -> Option<LocalConfig> {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return None;
    };

    if let Err(e) = set_current_dir(&local_dir) {
        eprintln!(
            "{}",
            t!(
                "jv.fail.std.set_current_dir",
                dir = local_dir.display(),
                error = e
            )
        );
        return None;
    }

    let Ok(local_config) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return None;
    };

    if !local_config.stained() {
        eprintln!("{}", md(t!("jv.fail.not_stained")));
        return None;
    }

    Some(local_config)
}

/// Build action pool and context for upstream communication
/// Returns Some((ActionPool, ActionContext)) if successful, None otherwise
async fn build_pool_and_ctx(
    local_config: &LocalConfig,
) -> Option<(ActionPool, ActionContext, Receiver<String>)> {
    let pool = client_registry::client_action_pool();
    let upstream = local_config.upstream_addr();

    let instance = connect(upstream).await?;

    // Build context and insert instance
    let mut ctx = ActionContext::local().insert_instance(instance);

    // Build channel for communication
    let (tx, rx) = mpsc::channel::<String>(100);
    ctx.insert_arc_data(Arc::new(tx));

    Some((pool, ctx, rx))
}

/// Sort paths in a vector of strings.
/// Paths are strings with structure A/B/C/D/E.
/// Paths with deeper levels (more '/' segments) are sorted first, followed by paths with shallower levels.
/// Within the same level, paths are sorted based on the first letter or digit encountered, with the order A-Z > a-z > 0-9.
fn sort_paths(paths: &mut Vec<String>) {
    quick_sort_with_cmp(paths, false, |a, b| {
        let depth_a = a.matches('/').count();
        let depth_b = b.matches('/').count();

        if depth_a != depth_b {
            return if depth_a > depth_b { -1 } else { 1 };
        }
        a.cmp(b) as i32
    });
}

/// Get paths that exist in the Cached Sheet under the current directory
fn mapping_names_here(
    current_dir: &PathBuf,
    local_dir: &PathBuf,
    cached_sheet: &SheetData,
) -> std::collections::BTreeMap<String, Option<SheetMappingMetadata>> {
    let Ok(relative_path) = current_dir.strip_prefix(local_dir) else {
        return std::collections::BTreeMap::new();
    };

    // Collect files directly under current directory
    let files_here: std::collections::BTreeMap<String, Option<SheetMappingMetadata>> = cached_sheet
        .mapping()
        .iter()
        .filter_map(|(f, mapping)| {
            // Check if the file is directly under the current directory
            f.parent()
                .filter(|&parent| parent == relative_path)
                .and_then(|_| f.file_name())
                .and_then(|name| name.to_str())
                .map(|s| (s.to_string(), Some(mapping.clone())))
        })
        .collect();

    // Collect directories that appear in the mapping
    let mut dirs_set = std::collections::BTreeSet::new();

    for (f, _mapping) in cached_sheet.mapping().iter() {
        // Get all parent directories of the file relative to the current directory
        let mut current = f.as_path();

        while let Some(parent) = current.parent() {
            if parent == relative_path {
                // This is a parent directory, not the file itself
                if current != f.as_path() {
                    if let Some(dir_name) = current.file_name() {
                        if let Some(dir_str) = dir_name.to_str() {
                            dirs_set.insert(format!("{}/", dir_str));
                        }
                    }
                }
                break;
            }
            current = parent;
        }
    }

    // Filter out directories that are actually files
    let filtered_dirs: std::collections::BTreeMap<String, Option<SheetMappingMetadata>> = dirs_set
        .into_iter()
        .filter(|dir_with_slash| {
            let dir_name = dir_with_slash.trim_end_matches('/');
            !files_here.contains_key(dir_name)
        })
        .map(|dir_name| (dir_name, None))
        .collect();

    // Combine results
    let mut result = files_here;
    result.extend(filtered_dirs);
    result
}

/// Trims the content, takes the first line, and truncates it to a display width of 24 characters.
/// If the display width exceeds 24, it truncates and adds "...".
fn truncate_first_line(content: String) -> String {
    let trimmed = content.trim();
    let first_line = trimmed.lines().next().unwrap_or("");
    let display_len = display_width(first_line);
    if display_len > 24 {
        let mut truncated = String::new();
        let mut current_len = 0;
        for ch in first_line.chars() {
            let ch_width = display_width(&ch.to_string());
            if current_len + ch_width > 24 {
                break;
            }
            truncated.push(ch);
            current_len += ch_width;
        }
        truncated.push_str("...");
        truncated
    } else {
        first_line.to_string()
    }
}
