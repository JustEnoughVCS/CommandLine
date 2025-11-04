use std::{env::current_dir, net::SocketAddr, path::PathBuf, process::exit};

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
                SetUpstreamVaultActionResult, UpdateToLatestInfoResult,
                proc_update_to_latest_info_action,
            },
            sheet_actions::{MakeSheetActionResult, proc_make_sheet_action},
        },
        constants::PORT,
        current::current_local_path,
        data::{
            local::{LocalWorkspace, config::LocalConfig, latest_info::LatestInfo},
            member::Member,
            user::UserDirectory,
        },
    },
};

use clap::{Parser, Subcommand, arg, command};
use just_enough_vcs::{
    utils::tcp_connection::error::TcpTargetError,
    vcs::{actions::local_actions::proc_set_upstream_vault_action, registry::client_registry},
};
use just_enough_vcs_cli::{
    data::compile_info::CompileInfo,
    utils::{
        input::confirm_hint_or, lang_selector::current_locales, md_colored::md, socket_addr_helper,
    },
};
use rust_i18n::{set_locale, t};
use tokio::{fs, net::TcpSocket};

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
}

#[derive(Parser, Debug)]
struct VersionArgs {
    #[arg(short = 'C', long = "compile-info")]
    compile_info: bool,
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
    #[command(alias = "mvkey", alias = "mvk")]
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

    /// Sheet name
    sheet_name: String,
}

#[derive(Parser, Debug)]
struct CreateWorkspaceArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Workspace directory path
    path: PathBuf,

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
    upstream: String,

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
        println!("{}", md(t!("jv.fail.parse.parser_failed")));
        return;
    };

    match parser.command {
        JustEnoughVcsWorkspaceCommand::Version(version_args) => {
            let compile_info = CompileInfo::default();
            println!(
                "{}",
                md(t!("jv.version.header", version = compile_info.cli_version))
            );

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
        JustEnoughVcsWorkspaceCommand::Sheets => jv_sheet_list(SheetListArgs { help: false }).await,
    }
}

async fn jv_create(args: CreateWorkspaceArgs) {
    let path = args.path;

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
    todo!()
}

async fn jv_sheet_list(_args: SheetListArgs) {
    let Some(_local_dir) = current_local_path() else {
        eprintln!("{}", t!("jv.fail.workspace_not_found").trim());
        return;
    };

    let Ok(latest_info) = LatestInfo::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let Ok(local_cfg) = LocalConfig::read().await else {
        eprintln!("{}", md(t!("jv.fail.read_cfg")));
        return;
    };

    let mut your_sheet_counts = 0;
    let mut other_sheet_counts = 0;

    // Print your sheets
    {
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
        println!();
    }

    // Print other sheets
    {
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
        println!();
    }

    // Print tips
    {
        if your_sheet_counts > 0 {
            println!("{}", md(t!("jv.success.sheet.list.tip_has_sheet")));
        } else {
            println!("{}", md(t!("jv.success.sheet.list.tip_no_sheet")));
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

    match local_cfg.use_sheet(args.sheet_name).await {
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

    let local_config = match precheck().await {
        Some(config) => config,
        None => return,
    };

    let (pool, ctx) = match build_pool_and_ctx(&local_config).await {
        Some(result) => result,
        None => return,
    };

    match proc_make_sheet_action(&pool, ctx, sheet_name.clone()).await {
        Ok(r) => match r {
            MakeSheetActionResult::Success => {
                eprintln!(
                    "{}",
                    md(t!("jv.result.sheet.make.success", name = sheet_name))
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
                println!(
                    "{}",
                    md(t!("jv.result.sheet.make.sheet_creation_failed", err = e))
                )
            }
            MakeSheetActionResult::Unknown => todo!(),
        },
        Err(e) => handle_err(e),
    }
}

async fn jv_sheet_drop(_args: SheetDropArgs) {
    todo!()
}

async fn jv_track(_args: TrackFileArgs) {
    todo!()
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
            eprintln!("{}", t!("jv.fail.account.add", account = args.account_name));
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

async fn jv_account_list(user_dir: UserDirectory, _args: AccountListArgs) {
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

    local_cfg.set_current_account(member.id());

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
    match fs::rename(
        args.key_path,
        user_dir.account_private_key_path(&args.account_name),
    )
    .await
    {
        Ok(_) => println!("{}", t!("jv.success.account.move_key")),
        Err(_) => eprintln!("{}", t!("jv.fail.account.move_key")),
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
                println!("{}", md(t!("jv.result.common.authroize_failed", err = e)))
            }
        },
    }
}

async fn jv_direct(args: DirectArgs) {
    if !args.confirm {
        println!(
            "{}",
            t!("jv.confirm.direct", upstream = args.upstream).trim()
        );
        confirm_hint_or(t!("common.confirm"), || exit(1)).await;
    }

    let pool = client_registry::client_action_pool();
    let upstream = match socket_addr_helper::get_socket_addr(&args.upstream, PORT).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!(
                "{}",
                md(t!(
                    "jv.fail.parse.str_to_sockaddr",
                    str = &args.upstream.trim(),
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
                )
            }
            SetUpstreamVaultActionResult::Redirected => {
                println!(
                    "{}",
                    md(t!("jv.result.direct.redirected", upstream = upstream))
                )
            }
            SetUpstreamVaultActionResult::AlreadyStained => {
                eprintln!("{}", md(t!("jv.result.direct.already_stained")))
            }
            SetUpstreamVaultActionResult::AuthorizeFailed(e) => {
                println!("{}", md(t!("jv.result.common.authroize_failed", err = e)))
            }
            SetUpstreamVaultActionResult::RedirectFailed(e) => {
                println!("{}", md(t!("jv.result.direct.redirect_failed", err = e)))
            }
            SetUpstreamVaultActionResult::SameUpstream => {
                println!("{}", md(t!("jv.result.direct.same_upstream")))
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
            md(t!("jv.warn.unstain", upstream = local_cfg.vault_addr()))
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

async fn jv_docs(_args: DocsArgs) {
    todo!()
}

pub fn handle_err(err: TcpTargetError) {
    eprintln!("{}", md(t!("jv.fail.from_just_version_control", err = err)))
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

// Build action pool and context for upstream communication
// Returns Some((ActionPool, ActionContext)) if successful, None otherwise
async fn build_pool_and_ctx(local_config: &LocalConfig) -> Option<(ActionPool, ActionContext)> {
    let pool = client_registry::client_action_pool();
    let upstream = local_config.upstream_addr();

    let instance = connect(upstream).await?;

    let ctx = ActionContext::local().insert_instance(instance);
    Some((pool, ctx))
}
