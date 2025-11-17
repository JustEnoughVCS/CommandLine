use colored::Colorize;
use just_enough_vcs::{
    system::action_system::{action::ActionContext, action_pool::ActionPool},
    utils::{
        cfg_file::config::ConfigFile,
        string_proc::{self, snake_case},
        tcp_connection::instance::ConnectionInstance,
    },
    vcs::{
        actions::{
            local_actions::{
                SetUpstreamVaultActionResult, SyncCachedSheetFailReason, UpdateToLatestInfoResult,
                proc_update_to_latest_info_action,
            },
            sheet_actions::{
                DropSheetActionResult, MakeSheetActionResult, proc_drop_sheet_action,
                proc_make_sheet_action,
            },
            virtual_file_actions::{
                CreateTaskResult, TrackFileActionArguments, TrackFileActionResult,
                proc_track_file_action,
            },
        },
        constants::{
            CLIENT_FILE_LATEST_INFO, CLIENT_FILE_WORKSPACE, CLIENT_FOLDER_WORKSPACE_ROOT_NAME,
            PORT, REF_SHEET_NAME,
        },
        current::{current_doc_dir, current_local_path},
        data::{
            local::{
                LocalWorkspace, cached_sheet::CachedSheet, config::LocalConfig,
                file_status::AnalyzeResult, latest_info::LatestInfo,
                local_files::get_relative_paths, member_held::MemberHeld,
            },
            member::{Member, MemberId},
            user::UserDirectory,
        },
        docs::{ASCII_YIZI, document, documents},
    },
};
use std::{
    env::{current_dir, set_current_dir},
    net::SocketAddr,
    path::PathBuf,
    process::exit,
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
        display::{SimpleTable, md, size_str},
        env::current_locales,
        fs::move_across_partitions,
        input::{confirm_hint, confirm_hint_or, input_with_editor},
        socket_addr_helper,
        sort::quick_sort_with_cmp,
    },
};
use rust_i18n::{set_locale, t};
use tokio::{fs, net::TcpSocket, time::Instant};

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
    Move(MoveFileArgs),

    /// Export files to other worksheet
    #[command(alias = "out")]
    Export(ExportFileArgs),

    /// Import files from reference sheet or import area
    #[command(alias = "in")]
    Import(ImportFileArgs),

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
    #[command(name = "_ip_history")]
    HistoryIpAddress,
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
}

#[derive(Parser, Debug)]
struct StatusArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct AccountAddArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Account name
    account_name: String,
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
struct TrackFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Track files
    track_files: Option<Vec<PathBuf>>,

    /// Commit - Editor mode
    #[arg(short, long)]
    work: bool,

    /// Commit - Text mode
    #[arg(short, long)]
    msg: bool,
}

#[derive(Parser, Debug)]
struct HoldFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct ThrowFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct MoveFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct ExportFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct ImportFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct UpdateArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
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

#[tokio::main]
async fn main() {
    // Init i18n
    set_locale(&current_locales());

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let Ok(parser) = JustEnoughVcsWorkspace::try_parse() else {
        eprintln!("{}", md(t!("jv.fail.parse.parser_failed")).red());

        // Tips
        // Guide to create
        {
            // Check if workspace exist
            let Some(local_dir) = current_local_path() else {
                println!();
                println!("{}", t!("jv.tip.not_workspace").trim().yellow());
                return;
            };

            // Check if account list is not empty
            let Some(dir) = UserDirectory::current_doc_dir() else {
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
            if let Some(local_cfg) = LocalConfig::read().await.ok() {
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
            }

            // Outdated
            let Ok(latest_info) =
                LatestInfo::read_from(local_dir.join(CLIENT_FILE_LATEST_INFO)).await
            else {
                return;
            };
            if let Some(instant) = latest_info.update_instant {
                let now = Instant::now();
                let duration = now.duration_since(instant);
                if duration.as_secs() > 60 * 15 {
                    // More than 15 minutes
                    let hours = duration.as_secs() / 3600;
                    let minutes = (duration.as_secs() % 3600) / 60;
                    println!();
                    println!(
                        "{}",
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
            if version_args.without_banner {
                println!(
                    "{}",
                    md(t!("jv.version.header", version = compile_info.cli_version))
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
                                t!("common.word.version"),
                                &compile_info.cli_version,
                                &compile_info.date
                            )
                        )
                        .replace("{banner_line_3}", "")
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
                        build_toolchain = compile_info.toolchain
                    ))
                );
            }
        }

        JustEnoughVcsWorkspaceCommand::Help => {
            println!("{}", md(t!("jv.help")));
        }

        JustEnoughVcsWorkspaceCommand::Account(account_manage) => {
            let user_dir = match UserDirectory::current_doc_dir() {
                Some(dir) => dir,
                None => {
                    eprintln!("{}", t!("jv.fail.account.no_user_dir").red());
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
        JustEnoughVcsWorkspaceCommand::Sheet(sheet_manage) => match sheet_manage {
            SheetManage::Help => {
                println!("{}", md(t!("jv.sheet")));
                return;
            }
            SheetManage::List(sheet_list_args) => jv_sheet_list(sheet_list_args).await,
            SheetManage::Use(sheet_use_args) => jv_sheet_use(sheet_use_args).await,
            SheetManage::Exit(sheet_exit_args) => jv_sheet_exit(sheet_exit_args).await,
            SheetManage::Make(sheet_make_args) => jv_sheet_make(sheet_make_args).await,
            SheetManage::Drop(sheet_drop_args) => jv_sheet_drop(sheet_drop_args).await,
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
        JustEnoughVcsWorkspaceCommand::Export(export_file_args) => {
            if export_file_args.help {
                println!("{}", md(t!("jv.export")));
                return;
            }
            jv_export(export_file_args).await;
        }
        JustEnoughVcsWorkspaceCommand::Import(import_file_args) => {
            if import_file_args.help {
                println!("{}", md(t!("jv.import")));
                return;
            }
            jv_import(import_file_args).await;
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
            jv_sheet_exit(SheetExitArgs { help: false }).await;
        }
        JustEnoughVcsWorkspaceCommand::Use(use_args) => {
            jv_sheet_exit(SheetExitArgs { help: false }).await;
            jv_sheet_use(SheetUseArgs {
                help: false,
                sheet_name: use_args.sheet_name,
            })
            .await;
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
            let user_dir = match UserDirectory::current_doc_dir() {
                Some(dir) => dir,
                None => {
                    eprintln!("{}", t!("jv.fail.account.no_user_dir").red());
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
        JustEnoughVcsWorkspaceCommand::As(args) => {
            let user_dir = match UserDirectory::current_doc_dir() {
                Some(dir) => dir,
                None => {
                    eprintln!("{}", t!("jv.fail.account.no_user_dir").red());
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

            let user_dir = match UserDirectory::current_doc_dir() {
                Some(dir) => dir,
                None => {
                    eprintln!("{}", t!("jv.fail.account.no_user_dir").red());
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

            jv_update(UpdateArgs { help: false }).await;
        }
        JustEnoughVcsWorkspaceCommand::HistoryIpAddress => {
            get_recent_ip_address()
                .await
                .iter()
                .for_each(|ip| println!("{}", ip));
        }
    }
}

async fn jv_create(args: CreateWorkspaceArgs) {
    let Some(path) = args.path else {
        println!("{}", md(t!("jv.create")));
        return;
    };

    if !args.force && path.exists() && !is_directory_empty(&path).await {
        eprintln!("{}", t!("jv.fail.init_create_dir_not_empty").trim().red());
        return;
    }

    match LocalWorkspace::setup_local_workspace(path).await {
        Ok(_) => {
            println!("{}", t!("jv.success.create"));
        }
        Err(e) => {
            eprintln!("{}", t!("jv.fail.create", error = e.to_string()).red());
        }
    }
}

async fn jv_init(args: InitWorkspaceArgs) {
    let path = match current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!(
                "{}",
                t!("jv.fail.get_current_dir", error = e.to_string()).red()
            );
            return;
        }
    };

    if !args.force && path.exists() && !is_directory_empty(&path).await {
        eprintln!("{}", t!("jv.fail.init_create_dir_not_empty").trim().red());
        return;
    }

    match LocalWorkspace::setup_local_workspace(path).await {
        Ok(_) => {
            println!("{}", t!("jv.success.init"));
        }
        Err(e) => {
            eprintln!("{}", t!("jv.fail.init", error = e.to_string()).red());
        }
    }
}

async fn is_directory_empty(path: &PathBuf) -> bool {
    match fs::read_dir(path).await {
        Ok(mut entries) => entries.next_entry().await.unwrap().is_none(),
        Err(_) => false,
    }
}

async fn jv_here(_args: HereArgs) {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
        return;
    };

    let Ok(latest_info) = LatestInfo::read_from(local_dir.join(CLIENT_FILE_LATEST_INFO)).await
    else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    let Ok(local_cfg) = LocalConfig::read_from(local_dir.join(CLIENT_FILE_WORKSPACE)).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    // Print path information
    let sheet_name = if let Some(sheet_name) = local_cfg.sheet_in_use() {
        sheet_name.to_string()
    } else {
        "".to_string()
    };

    let path = match current_dir() {
        Ok(path) => path,
        Err(_) => {
            eprintln!("{}", t!("jv.fail.get_current_dir").red());
            return;
        }
    };

    let local_dir = match current_local_path() {
        Some(path) => path,
        None => {
            eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
            return;
        }
    };

    let relative_path = match path.strip_prefix(&local_dir) {
        Ok(path) => path.display().to_string(),
        Err(_) => path.display().to_string(),
    };

    let duration_updated =
        Instant::now().duration_since(latest_info.update_instant.unwrap_or(Instant::now()));
    let minutes = duration_updated.as_secs() / 60;

    println!(
        "{}",
        t!(
            "jv.success.here.path_info",
            upstream = local_cfg.upstream_addr().to_string().cyan(),
            account = local_cfg.current_account().green(),
            sheet_name = sheet_name.yellow(),
            path = relative_path,
            minutes = minutes
        )
        .trim()
    );
    println!();

    // Print file info
    let mut table = SimpleTable::new(vec![
        t!("jv.success.here.items.name"),
        t!("jv.success.here.items.version"),
        t!("jv.success.here.items.hold"),
        t!("jv.success.here.items.size"),
        t!("jv.success.here.items.editing"),
    ]);

    let mut dir_count = 0;
    let mut file_count = 0;
    let mut total_size = 0;

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
                let version = "-";
                let hold = "-";
                let editing = "-";

                if is_dir {
                    dir_count += 1;
                    table.push_item(vec![
                        format!("{}/", file_name.cyan()),
                        version.to_string(),
                        hold.to_string(),
                        size_str(size as usize),
                        editing.to_string(),
                    ]);
                } else {
                    file_count += 1;
                    table.push_item(vec![
                        file_name,
                        version.to_string(),
                        hold.to_string(),
                        size_str(size as usize),
                        editing.to_string(),
                    ]);
                }

                total_size += size;
            }
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
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim().red());
        return;
    };

    let Ok(local_cfg) = LocalConfig::read_from(local_dir.join(CLIENT_FILE_WORKSPACE)).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    let account = local_cfg.current_account();

    let Ok(member_held_path) = MemberHeld::held_file_path(&account) else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    let Ok(member_held) = MemberHeld::read_from(&member_held_path).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    let Some(sheet_name) = local_cfg.sheet_in_use().clone() else {
        eprintln!("{}", md(t!("jv.fail.status.no_sheet_in_use")).trim().red());
        return;
    };

    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_cfg) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim().red());
        return;
    };

    let Ok(local_sheet) = local_workspace.local_sheet(&account, &sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    let Ok(cached_sheet) = CachedSheet::cached_sheet_data(&sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    let Ok(analyzed) = AnalyzeResult::analyze_local_status(&local_workspace).await else {
        eprintln!("{}", md(t!("jv.fail.status.analyze")).trim().red());
        return;
    };

    println!(
        "{}",
        t!("jv.success.status.header", sheet_name = sheet_name)
    );

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
                    match member_held.file_holder(vfid) {
                        Some(holder) => holder == &account,
                        None => false,
                    }
                } else {
                    false
                }
            };

            let base_version_match = {
                if let Ok(mapping) = local_sheet.mapping_data(path) {
                    let ver = mapping.version_when_updated();
                    if let Some(cached) = cached_sheet.mapping().get(path) {
                        ver == &cached.version
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

    let has_struct_changes =
        !created_items.is_empty() || !lost_items.is_empty() || !moved_items.is_empty();
    let has_file_modifications = !modified_items.is_empty();

    if has_struct_changes {
        sort_paths(&mut created_items);
        sort_paths(&mut lost_items);
        sort_paths(&mut moved_items);
    }
    if has_file_modifications {
        sort_paths(&mut modified_items);
    }

    println!(
        "{}",
        md(t!(
            "jv.success.status.content",
            moved_items = if has_struct_changes {
                if moved_items.is_empty() {
                    "".to_string()
                } else {
                    moved_items.join("\n") + "\n"
                }
            } else {
                t!("jv.success.status.no_structure_changes")
                    .trim()
                    .to_string()
                    + "\n"
            },
            lost_items = if has_struct_changes {
                if lost_items.is_empty() {
                    "".to_string()
                } else {
                    lost_items.join("\n") + "\n"
                }
            } else {
                "".to_string()
            },
            created_items = if has_struct_changes {
                if created_items.is_empty() {
                    "".to_string()
                } else {
                    created_items.join("\n") + "\n"
                }
            } else {
                "".to_string()
            },
            modified_items = if has_file_modifications {
                if modified_items.is_empty() {
                    "".to_string()
                } else {
                    modified_items.join("\n") + "\n"
                }
            } else {
                t!("jv.success.status.no_file_modifications")
                    .trim()
                    .to_string()
            }
        ))
        .trim()
    );
}

async fn jv_sheet_list(args: SheetListArgs) {
    let Some(_local_dir) = current_local_path() else {
        if !args.raw {
            eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
        }
        return;
    };

    let Ok(latest_info) = LatestInfo::read().await else {
        if !args.raw {
            eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        }
        return;
    };

    let Ok(local_cfg) = LocalConfig::read().await else {
        if !args.raw {
            eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        }
        return;
    };

    let mut your_sheet_counts = 0;
    let mut other_sheet_counts = 0;

    if args.raw {
        // Print your sheets
        if !args.others && !args.all || !args.others {
            latest_info.my_sheets.iter().for_each(|s| println!("{}", s));
        }
        // Print other sheets
        if args.others || args.all {
            latest_info
                .other_sheets
                .iter()
                .for_each(|s| println!("{}", s.sheet_name));
        }
    } else {
        // Print your sheets
        if !args.others && !args.all || !args.others {
            println!("{}", md(t!("jv.success.sheet.list.your_sheet")));
            let in_use = local_cfg.sheet_in_use();
            for sheet in latest_info.my_sheets {
                if let Some(in_use) = in_use
                    && in_use == &sheet
                {
                    println!(
                        "{}",
                        md(t!(
                            "jv.success.sheet.list.your_sheet_item_use",
                            number = your_sheet_counts + 1,
                            name = sheet
                        ))
                    );
                } else {
                    println!(
                        "{}",
                        md(t!(
                            "jv.success.sheet.list.your_sheet_item",
                            number = your_sheet_counts + 1,
                            name = sheet
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
            for sheet in latest_info.other_sheets {
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
    let Some(_local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
        return;
    };

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    match local_cfg.use_sheet(args.sheet_name).await {
        Ok(_) => {
            let Ok(_) = LocalConfig::write(&local_cfg).await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim().red());
                return;
            };
        }
        Err(e) => {
            handle_err(e.into());
        }
    }
}

async fn jv_sheet_exit(_args: SheetExitArgs) {
    let Some(_local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
        return;
    };

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    match local_cfg.exit_sheet().await {
        Ok(_) => {
            let Ok(_) = LocalConfig::write(&local_cfg).await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim().red());
                return;
            };
        }
        Err(e) => {
            handle_err(e.into());
        }
    }
}

async fn jv_sheet_make(args: SheetMakeArgs) {
    let sheet_name = snake_case!(args.sheet_name);

    if sheet_name == REF_SHEET_NAME {
        eprintln!(
            "{}",
            t!("jv.confirm.sheet.make.restore_ref").trim().yellow()
        );
        return;
    }

    let local_config = match precheck().await {
        Some(config) => config,
        None => return,
    };

    let (pool, ctx) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    let latest_info = match LatestInfo::read().await {
        Ok(info) => info,
        Err(_) => {
            eprintln!("{}", t!("jv.fail.read_cfg").red());
            return;
        }
    };

    if latest_info
        .other_sheets
        .iter()
        .any(|sheet| sheet.sheet_name == sheet_name)
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
                eprintln!(
                    "{}",
                    md(t!("jv.result.common.authroize_failed", err = e)).red()
                )
            }
            MakeSheetActionResult::SheetAlreadyExists => {
                eprintln!(
                    "{}",
                    md(t!(
                        "jv.result.sheet.make.sheet_already_exists",
                        name = sheet_name
                    ))
                    .red()
                );
            }
            MakeSheetActionResult::SheetCreationFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.make.sheet_creation_failed", err = e)).red()
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

    let (pool, ctx) = match build_pool_and_ctx(&local_config).await {
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
                    md(t!("jv.result.sheet.drop.sheet_in_use", name = sheet_name)).red()
                )
            }
            DropSheetActionResult::AuthorizeFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.common.authroize_failed", err = e)).red()
                )
            }
            DropSheetActionResult::SheetNotExists => {
                eprintln!(
                    "{}",
                    md(t!(
                        "jv.result.sheet.drop.sheet_not_exists",
                        name = sheet_name
                    ))
                    .red()
                )
            }
            DropSheetActionResult::SheetDropFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.drop.sheet_drop_failed", err = e)).red()
                )
            }
            DropSheetActionResult::NoHolder => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.drop.no_holder", name = sheet_name)).red()
                )
            }
            DropSheetActionResult::NotOwner => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.drop.not_owner", name = sheet_name)).red()
                )
            }
            _ => {}
        },
        Err(e) => handle_err(e),
    }
}

async fn jv_track(args: TrackFileArgs) {
    let Some(track_files) = args.track_files else {
        println!("{}", md(t!("jv.track")));
        return;
    };

    let local_config = match precheck().await {
        Some(config) => config,
        None => {
            return;
        }
    };

    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
        return;
    };

    let Some(files) = get_relative_paths(local_dir, track_files).await else {
        eprintln!(
            "{}",
            md(t!("jv.fail.track.parse_fail", param = "track_files")).red()
        );
        return;
    };

    if files.iter().len() < 1 {
        eprintln!("{}", md(t!("jv.fail.track.no_selection")).red());
        return;
    };

    let (pool, ctx) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    match proc_track_file_action(
        &pool,
        ctx,
        TrackFileActionArguments {
            relative_pathes: files.iter().cloned().collect(),
            display_progressbar: true,
        },
    )
    .await
    {
        Ok(result) => match result {
            TrackFileActionResult::Done {
                created,
                updated,
                synced,
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
            }
            TrackFileActionResult::AuthorizeFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.common.authroize_failed", err = e)).red()
                )
            }
            TrackFileActionResult::StructureChangesNotSolved => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.track.structure_changes_not_solved")).red()
                )
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
                        .red()
                    )
                }
                CreateTaskResult::SheetNotFound(sheet) => {
                    eprintln!(
                        "{}",
                        md(t!(
                            "jv.result.track.create_failed.sheet_not_found",
                            name = sheet
                        ))
                        .red()
                    )
                }
            },
            TrackFileActionResult::UpdateTaskFailed(update_task_result) => todo!(),
            TrackFileActionResult::SyncTaskFailed(sync_task_result) => todo!(),
        },
        Err(e) => handle_err(e),
    }
}

async fn jv_hold(_args: HoldFileArgs) {
    todo!()
}

async fn jv_throw(_args: ThrowFileArgs) {
    todo!()
}

async fn jv_move(_args: MoveFileArgs) {
    todo!()
}

async fn jv_export(_args: ExportFileArgs) {
    todo!()
}

async fn jv_import(_args: ImportFileArgs) {
    todo!()
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
            eprintln!(
                "{}",
                t!("jv.fail.account.add", account = args.account_name).red()
            );
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
                t!("jv.fail.account.remove", account = args.account_name).red()
            );
        }
    }
}

async fn jv_account_list(user_dir: UserDirectory, args: AccountListArgs) {
    if args.raw {
        let Ok(account_ids) = user_dir.account_ids() else {
            return;
        };
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
            eprintln!("{}", t!("jv.fail.account.list").red());
        }
    }
}

async fn jv_account_as(user_dir: UserDirectory, args: SetLocalWorkspaceAccountArgs) {
    // Account exist
    let Ok(member) = user_dir.account(&args.account_name).await else {
        eprintln!(
            "{}",
            t!("jv.fail.account.not_found", account = args.account_name).red()
        );
        return;
    };

    let Some(_local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
        return;
    };

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    if let Err(_) = local_cfg.set_current_account(member.id()) {
        eprintln!("{}", md(t!("jv.fail.account.as")).red());
        return;
    };

    let Ok(_) = LocalConfig::write(&local_cfg).await else {
        eprintln!("{}", t!("jv.fail.write_cfg").trim().red());
        return;
    };

    println!(
        "{}",
        t!("jv.success.account.as", account = member.id()).trim()
    );
}

async fn jv_account_move_key(user_dir: UserDirectory, args: MoveKeyToAccountArgs) {
    // Key file exist
    if !args.key_path.exists() {
        eprintln!(
            "{}",
            t!("jv.fail.path_not_found", path = args.key_path.display()).red()
        );
        return;
    }

    // Account exist
    let Ok(_member) = user_dir.account(&args.account_name).await else {
        eprintln!(
            "{}",
            t!("jv.fail.account.not_found", account = args.account_name).red()
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
        Err(_) => eprintln!("{}", t!("jv.fail.account.move_key").red()),
    }
}

async fn jv_update(_update_file_args: UpdateArgs) {
    let local_config = match precheck().await {
        Some(config) => config,
        None => return,
    };

    let (pool, ctx) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    match proc_update_to_latest_info_action(&pool, ctx, ()).await {
        Err(e) => handle_err(e),
        Ok(result) => match result {
            UpdateToLatestInfoResult::Success => {
                println!("{}", md(t!("jv.result.update.success")));
            }
            UpdateToLatestInfoResult::AuthorizeFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.common.authroize_failed", err = e)).red()
                )
            }
            UpdateToLatestInfoResult::SyncCachedSheetFail(sync_cached_sheet_fail_reason) => {
                match sync_cached_sheet_fail_reason {
                    SyncCachedSheetFailReason::PathAlreadyExist(path_buf) => {
                        eprintln!(
                            "{}",
                            md(t!(
                                "jv.result.update.fail.sync_cached_sheet_fail.path_already_exist",
                                path = path_buf.display()
                            ))
                            .red()
                        );
                    }
                }
            }
        },
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
                .red()
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
                eprintln!("{}", md(t!("jv.result.direct.already_stained")).red())
            }
            SetUpstreamVaultActionResult::AuthorizeFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.common.authroize_failed", err = e)).red()
                )
            }
            SetUpstreamVaultActionResult::RedirectFailed(e) => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.direct.redirect_failed", err = e)).red()
                )
            }
            SetUpstreamVaultActionResult::SameUpstream => {
                eprintln!("{}", md(t!("jv.result.direct.same_upstream")).red())
            }
            _ => {}
        },
    };
}

async fn jv_unstain(args: UnstainArgs) {
    let Some(_local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
        return;
    };

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return;
    };

    if !local_cfg.stained() {
        eprintln!("{}", md(t!("jv.fail.unstain")).red());
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
        eprintln!("{}", t!("jv.fail.write_cfg").trim().red());
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
            md(t!("jv.fail.docs.not_found", docs_name = docs_name)).red()
        );
        return;
    };

    if args.direct {
        println!("{}", document.trim());
    } else {
        let Some(doc_dir) = current_doc_dir() else {
            eprintln!(
                "{}",
                md(t!("jv.fail.docs.no_doc_dir", docs_name = docs_name)).red()
            );
            return;
        };
        let file = doc_dir.join("DOCS.MD");
        if let Err(e) = input_with_editor(document, file, "").await {
            eprintln!(
                "{}",
                md(t!(
                    "jv.fail.docs.open_editor",
                    err = e,
                    docs_name = docs_name
                ))
                .red()
            );
        }
    }
}

pub fn handle_err(err: TcpTargetError) {
    eprintln!("{}", md(t!("jv.fail.from_core", err = err)).red())
}

async fn connect(upstream: SocketAddr) -> Option<ConnectionInstance> {
    // Create Socket
    let socket = if upstream.is_ipv4() {
        match TcpSocket::new_v4() {
            Ok(socket) => socket,
            Err(_) => {
                eprintln!("{}", t!("jv.fail.create_socket").trim().red());
                return None;
            }
        }
    } else {
        match TcpSocket::new_v6() {
            Ok(socket) => socket,
            Err(_) => {
                eprintln!("{}", t!("jv.fail.create_socket").trim().red());
                return None;
            }
        }
    };

    // Connect
    let Ok(stream) = socket.connect(upstream).await else {
        eprintln!("{}", t!("jv.fail.connection_failed").trim().red());
        return None;
    };

    Some(ConnectionInstance::from(stream))
}

// Check if the workspace is stained and has a valid configuration
// Returns LocalConfig if valid, None otherwise
async fn precheck() -> Option<LocalConfig> {
    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim().red());
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
            .red()
        );
        return None;
    }

    let Ok(local_config) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")).red());
        return None;
    };

    if !local_config.stained() {
        eprintln!("{}", md(t!("jv.fail.not_stained")).red());
        return None;
    }

    Some(local_config)
}

// Build action pool and context for upstream communication
// Returns Some((ActionPool, ActionContext)) if successful, None otherwise
async fn build_pool_and_ctx(local_config: &LocalConfig) -> Option<(ActionPool, ActionContext)> {
    let pool = client_registry::client_action_pool();
    let upstream = local_config.upstream_addr();

    let instance = connect(upstream).await?;

    let ctx = ActionContext::local().insert_instance(instance);
    Some((pool, ctx))
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
