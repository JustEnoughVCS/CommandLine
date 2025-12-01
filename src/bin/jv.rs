use colored::Colorize;
use just_enough_vcs::{
    system::action_system::{action::ActionContext, action_pool::ActionPool},
    utils::{
        cfg_file::config::ConfigFile,
        data_struct::dada_sort::quick_sort_with_cmp,
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
            CLIENT_PATH_WORKSPACE_ROOT, PORT, REF_SHEET_NAME,
        },
        current::{correct_current_dir, current_cfg_dir, current_local_path},
        data::{
            local::{
                LocalWorkspace, align::AlignTasks, cached_sheet::CachedSheet, config::LocalConfig,
                file_status::AnalyzeResult, latest_file_data::LatestFileData,
                latest_info::LatestInfo, local_files::get_relative_paths,
                vault_modified::check_vault_modified,
            },
            member::{Member, MemberId},
            sheet::SheetData,
            user::UserDirectory,
            vault::virtual_file::VirtualFileVersion,
        },
        docs::{ASCII_YIZI, document, documents},
    },
};
use std::{
    collections::{HashMap, HashSet},
    env::{current_dir, set_current_dir},
    net::SocketAddr,
    path::PathBuf,
    process::exit,
    str::FromStr,
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
        env::{current_locales, enable_auto_update},
        fs::move_across_partitions,
        input::{confirm_hint, confirm_hint_or, input_with_editor, show_in_pager},
        socket_addr_helper,
    },
};
use rust_i18n::{set_locale, t};
use tokio::{fs, net::TcpSocket, process::Command, time::Instant};

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

    /// Track files
    track_files: Option<Vec<PathBuf>>,

    /// Commit - Description
    #[arg(short, long)]
    desc: Option<String>,

    /// Commit - Description
    #[arg(short, long)]
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

    /// Hold files
    hold_files: Option<Vec<PathBuf>>,

    /// Show fail details
    #[arg(short = 'd', long = "details")]
    show_fail_details: bool,

    /// Skip failed items
    #[arg(short = 'S', long)]
    skip_failed: bool,
}

#[derive(Parser, Debug)]
struct ThrowFileArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Throw files
    throw_files: Option<Vec<PathBuf>>,

    /// Show fail details
    #[arg(short = 'd', long = "details")]
    show_fail_details: bool,

    /// Skip failed items
    #[arg(short = 'S', long)]
    skip_failed: bool,
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

#[tokio::main]
async fn main() {
    // Init i18n
    set_locale(&current_locales());

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    // Auto update
    if enable_auto_update() && check_vault_modified().await {
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
        JustEnoughVcsWorkspaceCommand::GetHistoryIpAddress => {
            get_recent_ip_address()
                .await
                .iter()
                .for_each(|ip| println!("{}", ip));
        }
        JustEnoughVcsWorkspaceCommand::GetWorkspaceDir => {
            if let Some(local_dir) = current_local_path() {
                println!("{}", local_dir.display())
            };
        }
        JustEnoughVcsWorkspaceCommand::GetCurrentAccount => {
            if let Ok(local_config) = LocalConfig::read().await {
                println!("{}", local_config.current_account())
            };
        }
        JustEnoughVcsWorkspaceCommand::GetCurrentUpstream => {
            if let Ok(local_config) = LocalConfig::read().await {
                println!("{}", local_config.upstream_addr())
            };
        }
        JustEnoughVcsWorkspaceCommand::GetCurrentSheet => {
            if let Ok(local_config) = LocalConfig::read().await {
                println!(
                    "{}",
                    local_config.sheet_in_use().clone().unwrap_or_default()
                )
            };
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

async fn jv_here(_args: HereArgs) {
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

    let remote_files = mapping_names_here(&path, &local_dir, &cached_sheet);

    let duration_updated =
        Instant::now().duration_since(latest_info.update_instant.unwrap_or(Instant::now()));
    let minutes = duration_updated.as_secs() / 60;

    println!(
        "{}",
        t!(
            "jv.success.here.path_info",
            upstream = local_cfg.upstream_addr().to_string(),
            account = local_cfg.current_account(),
            sheet_name = sheet_name.yellow(),
            path = relative_path,
            minutes = minutes
        )
        .trim()
    );

    // Print file info
    let mut table = SimpleTable::new(vec![
        t!("jv.success.here.items.editing"),
        t!("jv.success.here.items.holder"),
        t!("jv.success.here.items.size"),
        t!("jv.success.here.items.version"),
        t!("jv.success.here.items.name"),
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
                let mut version = "-".to_string();
                let mut hold = "-".to_string();
                let mut editing = "-".to_string();

                if is_dir {
                    // Directory
                    // Add directory count
                    dir_count += 1;

                    // Add directory item
                    table.insert_item(
                        0,
                        vec![
                            editing.to_string(),
                            hold.to_string(),
                            "-".to_string(),
                            version.to_string(),
                            t!(
                                "jv.success.here.append_info.name",
                                name = format!("{}/", file_name.cyan())
                            )
                            .trim()
                            .to_string(),
                        ],
                    );
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

                        // Version status
                        if let Some(latest_version) = latest_file_data.file_version(&id) {
                            let local_version = local_sheet.mapping_data(&current_path);
                            if let Ok(local_mapping) = local_version {
                                let local_version = local_mapping.version_when_updated();
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

                    table.push_item(vec![
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
                    ]);
                }

                // Total Size
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

    remote_files.iter().for_each(|f| println!("{}", f));
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

    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_cfg) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    let Ok(local_sheet) = local_workspace.local_sheet(&account, &sheet_name).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(analyzed) = AnalyzeResult::analyze_local_status(&local_workspace).await else {
        eprintln!("{}", md(t!("jv.fail.status.analyze")).trim());
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

    // Calculate duration since last update
    let update_instant = latest_info.update_instant.unwrap_or(Instant::now());
    let duration = Instant::now().duration_since(update_instant);
    let h = duration.as_secs() / 3600;
    let m = (duration.as_secs() % 3600) / 60;
    let s = duration.as_secs() % 60;

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
                    modified_items.join("\n")
                }
            } else {
                t!("jv.success.status.no_file_modifications")
                    .trim()
                    .to_string()
            },
            h = h,
            m = m,
            s = s
        ))
        .trim()
    );
}

async fn jv_sheet_list(args: SheetListArgs) {
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
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

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

async fn jv_sheet_exit(_args: SheetExitArgs) {
    let Some(_local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Ok(mut local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    match local_cfg.exit_sheet().await {
        Ok(_) => {
            let Ok(_) = LocalConfig::write(&local_cfg).await else {
                eprintln!("{}", t!("jv.fail.write_cfg").trim());
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
        .other_sheets
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

    let Ok(local_cfg) = LocalConfig::read_from(local_dir.join(CLIENT_FILE_WORKSPACE)).await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };
    let Some(local_workspace) = LocalWorkspace::init_current_dir(local_cfg) else {
        eprintln!("{}", md(t!("jv.fail.workspace_not_found")).trim());
        return;
    };

    let Ok(analyzed) = AnalyzeResult::analyze_local_status(&local_workspace).await else {
        eprintln!("{}", md(t!("jv.fail.status.analyze")).trim());
        return;
    };

    let align_tasks = AlignTasks::from_analyze_result(analyzed);

    // No task input, list mode
    let Some(task) = args.task else {
        // Raw output
        if args.raw {
            if args.list_all {
                align_tasks.created.iter().for_each(|i| println!("{}", i.0));
                align_tasks.moved.iter().for_each(|i| println!("{}", i.0));
                align_tasks.lost.iter().for_each(|i| println!("{}", i.0));
                return;
            }
            if args.list_created {
                align_tasks.created.iter().for_each(|i| println!("{}", i.0));
                return;
            }
            if args.list_unsolved {
                align_tasks.moved.iter().for_each(|i| println!("{}", i.0));
                align_tasks.lost.iter().for_each(|i| println!("{}", i.0));
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

        let mut empty_count = 0;

        if !align_tasks.created.is_empty() {
            align_tasks.created.iter().for_each(|(n, p)| {
                table.push_item(vec![
                    format!("+ {}", n).green().to_string(),
                    p.display().to_string().green().to_string(),
                    "".to_string(),
                ]);
            });
        } else {
            empty_count += 1;
        }

        if !align_tasks.lost.is_empty() {
            align_tasks.lost.iter().for_each(|(n, p)| {
                table.push_item(vec![
                    format!("- {}", n).red().to_string(),
                    p.display().to_string().red().to_string(),
                    "".to_string(),
                ]);
            });
        } else {
            empty_count += 1;
        }

        if !align_tasks.moved.is_empty() {
            align_tasks.moved.iter().for_each(|(n, (rp, lp))| {
                table.push_item(vec![
                    format!("> {}", n).yellow().to_string(),
                    lp.display().to_string().yellow().to_string(),
                    rp.display().to_string(),
                ]);
            });
        } else {
            empty_count += 1;
        }

        if empty_count == 3 {
            println!("{}", md(t!("jv.success.sheet.align.no_changes").trim()));
        } else {
            println!(
                "{}",
                md(t!("jv.success.sheet.align.list", tasks = table.to_string()))
            );
        }

        return;
    };

    let Some(to) = args.to else {
        eprintln!("{}", md(t!("jv.fail.sheet.align.no_direction")));
        return;
    };
}

async fn jv_track(args: TrackFileArgs) {
    let track_files = if let Some(files) = args.track_files.clone() {
        files
            .iter()
            .map(|f| current_dir().unwrap().join(f))
            .collect::<Vec<_>>()
    } else {
        println!("{}", md(t!("jv.track")));
        return;
    };

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

    let Some(local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Some(files) = get_relative_paths(&local_dir, &track_files).await else {
        eprintln!(
            "{}",
            md(t!("jv.fail.track.parse_fail", param = "track_files"))
        );
        return;
    };

    if files.iter().len() < 1 {
        eprintln!("{}", md(t!("jv.fail.track.no_selection")));
        return;
    };

    let (pool, ctx) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    let files = files.iter().cloned().collect();
    let update_info = get_update_info(local_workspace, &files, args).await;

    match proc_track_file_action(
        &pool,
        ctx,
        TrackFileActionArguments {
            relative_pathes: files,
            file_update_info: update_info,
            print_infos: true,
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
                    return Some((file.clone(), mapping_item.version.clone()));
                }
            }
            None
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
        table.push_item(vec![
            item.0.display().to_string(),
            item.1.to_string(),
            t!("editor.modified_line.content.arrow").trim().to_string(),
            " ".to_string(),
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

                        update_info.insert(path, (new_version.to_string(), description));
                    }
                }
            }
        }
    }

    update_info
}

async fn jv_hold(args: HoldFileArgs) {
    let hold_files = if let Some(files) = args.hold_files.clone() {
        files
            .iter()
            .map(|f| current_dir().unwrap().join(f))
            .collect::<Vec<_>>()
    } else {
        println!("{}", md(t!("jv.hold")));
        return;
    };

    let _ = correct_current_dir();

    jv_change_edit_right(
        hold_files,
        EditRightChangeBehaviour::Hold,
        args.show_fail_details,
        args.skip_failed,
    )
    .await;
}

async fn jv_throw(args: ThrowFileArgs) {
    let throw_files = if let Some(files) = args.throw_files.clone() {
        files
            .iter()
            .map(|f| current_dir().unwrap().join(f))
            .collect::<Vec<_>>()
    } else {
        println!("{}", md(t!("jv.throw")));
        return;
    };

    let _ = correct_current_dir();

    jv_change_edit_right(
        throw_files,
        EditRightChangeBehaviour::Throw,
        args.show_fail_details,
        args.skip_failed,
    )
    .await;
}

async fn jv_change_edit_right(
    files: Vec<PathBuf>,
    behaviour: EditRightChangeBehaviour,
    show_fail_details: bool,
    mut skip_failed: bool,
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

    // Precheck and filter
    let Some(filtered_files) = get_relative_paths(&local_dir, &files).await else {
        eprintln!(
            "{}",
            md(t!("jv.fail.track.parse_fail", param = "track_files"))
        );
        return;
    };
    let num = filtered_files.iter().len();
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

    for file in filtered_files {
        let full_path = local_dir.join(&file);

        // File exists
        if !full_path.exists() {
            let reason = t!(
                "jv.fail.change_edit_right.check_fail_item",
                path = file.display(),
                reason = t!("jv.fail.change_edit_right.check_fail_reason.not_found_in_local")
            )
            .trim()
            .to_string();

            if !handle_validation_failure(show_fail_details, &mut details, &mut failed, num, reason)
            {
                return;
            }
            continue;
        }

        // Mapping exists
        if !cached_sheet.mapping().contains_key(&file) {
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
        }

        // Not tracked
        let Ok(local_mapping) = local_sheet.mapping_data(&file) else {
            let reason = t!(
                "jv.fail.change_edit_right.check_fail_item",
                path = file.display(),
                reason = t!("jv.fail.change_edit_right.check_fail_reason.not_a_tracked_file")
            )
            .trim()
            .to_string();

            if !handle_validation_failure(show_fail_details, &mut details, &mut failed, num, reason)
            {
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

            if !handle_validation_failure(show_fail_details, &mut details, &mut failed, num, reason)
            {
                return;
            }
            continue;
        }

        // Hold validation
        let holder = latest_file_data.file_holder(vfid);
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

    let (pool, ctx) = match build_pool_and_ctx(&local_cfg).await {
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
            eprintln!("{}", t!("jv.fail.account.list"));
        }
    }
}

async fn jv_account_as(user_dir: UserDirectory, args: SetLocalWorkspaceAccountArgs) {
    // Account exist
    let Ok(member) = user_dir.account(&args.account_name).await else {
        eprintln!(
            "{}",
            t!("jv.fail.account.not_found", account = args.account_name)
        );
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

    let Ok(_) = LocalConfig::write(&local_cfg).await else {
        eprintln!("{}", t!("jv.fail.write_cfg").trim());
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

    let (pool, ctx) = match build_pool_and_ctx(&local_config).await {
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
/// Get paths that exist in the Cached Sheet under the current directory
fn mapping_names_here(
    current_dir: &PathBuf,
    local_dir: &PathBuf,
    cached_sheet: &SheetData,
) -> Vec<String> {
    let Ok(relative_path) = current_dir.strip_prefix(local_dir) else {
        return Vec::new();
    };

    // Collect files directly under current directory
    let files_here: Vec<String> = cached_sheet
        .mapping()
        .iter()
        .filter_map(|(f, _)| {
            // Check if the file is directly under the current directory
            f.parent()
                .filter(|&parent| parent == relative_path)
                .and_then(|_| f.file_name())
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .collect();

    // Collect directories that appear in the mapping
    let mut dirs_set = std::collections::HashSet::new();

    for (f, _) in cached_sheet.mapping().iter() {
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
    let filtered_dirs: Vec<String> = dirs_set
        .into_iter()
        .filter(|dir_with_slash| {
            let dir_name = dir_with_slash.trim_end_matches('/');
            !files_here.iter().any(|file_name| file_name == dir_name)
        })
        .collect();

    // Combine results
    let mut result = files_here;
    result.extend(filtered_dirs);
    result
}
