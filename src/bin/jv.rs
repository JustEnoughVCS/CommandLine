use std::{env::current_dir, net::SocketAddr, path::PathBuf, str::FromStr};

use just_enough_vcs::{
    utils::cfg_file::config::ConfigFile,
    vcs::{
        current::current_local_path,
        data::{
            local::{LocalWorkspace, config::LocalConfig},
            member::Member,
            user::UserDirectory,
        },
    },
};

use clap::{Parser, Subcommand, arg, command};
use just_enough_vcs::{
    system::action_system::action::ActionContext,
    utils::tcp_connection::error::TcpTargetError,
    vcs::{actions::local_actions::proc_set_upstream_vault_action, registry::client_registry},
};
use just_enough_vcs_cli::utils::{lang_selector::current_locales, md_colored::md};
use rust_i18n::{set_locale, t};
use tokio::fs;

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
    #[command(subcommand)]
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

    // Connection management
    /// Direct to an upstream vault and stain this workspace
    Direct(DirectArgs),

    /// DANGER ZONE : Unstain this workspace
    Unstain(UnstainArgs),

    // Other
    /// Query built-in documentation
    Docs(DocsArgs),
}

#[derive(Subcommand, Debug)]
enum AccountManage {
    /// Show help information
    #[command(alias = "--help", alias = "-h")]
    Help,

    /// Register a member to this computer
    Add(AccountAddArgs),

    /// Remove a account from this computer
    #[command(alias = "rm")]
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
struct DirectArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Upstream vault address
    upstream: String,
}

#[derive(Parser, Debug)]
struct UnstainArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct DocsArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[tokio::main]
async fn main() {
    // Init i18n
    set_locale(&current_locales());

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let Ok(parser) = JustEnoughVcsWorkspace::try_parse() else {
        println!("{}", md(t!("jv.help")));
        return;
    };

    match parser.command {
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

async fn jv_direct(args: DirectArgs) {
    let pool = client_registry::client_action_pool();
    let upstream = match socket_addr_helper::get_socket_addr(&args.upstream, PORT).await {
        Ok(result) => result,
        Err(_) => {
            eprintln!(
                "{}",
                md(t!(
                    "jv.fail.parse.str_to_sockaddr",
                    str = &args.upstream.trim()
                ))
            );
            return;
        }
    };
    let ctx = ActionContext::local();
    match proc_set_upstream_vault_action(&pool, ctx, upstream).await {
        Err(e) => handle_err(e),
        _ => {}
    };
}

async fn jv_unstain(_args: UnstainArgs) {
    todo!()
}

async fn jv_docs(_args: DocsArgs) {
    todo!()
}

pub fn handle_err(err: TcpTargetError) {
    let e: Option<(String, String, bool)> = match err {
        TcpTargetError::Io(err) => Some(fsio(err)),
        TcpTargetError::File(err) => Some(fsio(err)),

        TcpTargetError::Serialization(err) => Some(serialize(err)),

        TcpTargetError::Authentication(err) => Some(auth(err)),

        TcpTargetError::Network(err) => Some(connection(err)),
        TcpTargetError::Timeout(err) => Some(connection(err)),
        TcpTargetError::Protocol(err) => Some(connection(err)),

        _ => Some((
            err.to_string(),
            md(t!("jv.fail.action_operation_fail.type_other")),
            false,
        )),
    };

    if let Some((err_text, err_tip, has_tip)) = e {
        eprintln!(
            "{}\n{}",
            md(t!("jv.fail.action_operation_fail.main", err = err_text)),
            err_tip,
        );

        if has_tip {
            eprintln!(
                "{}",
                md(t!("jv.fail.action_operation_fail.info_contact_admin"))
            )
        }
    }
}

type ErrorText = String;
type ErrorTip = String;
type HasTip = bool;

fn fsio(err: String) -> (ErrorText, ErrorTip, HasTip) {
    (
        err,
        md(t!("jv.fail.action_operation_fail.type_fsio")).to_string(),
        true,
    )
}

fn serialize(err: String) -> (ErrorText, ErrorTip, HasTip) {
    (
        err,
        md(t!("jv.fail.action_operation_fail.type_serialize")).to_string(),
        true,
    )
}

fn auth(err: String) -> (ErrorText, ErrorTip, HasTip) {
    (
        err,
        md(t!("jv.fail.action_operation_fail.type_auth")).to_string(),
        true,
    )
}

fn connection(err: String) -> (ErrorText, ErrorTip, HasTip) {
    (
        err,
        md(t!("jv.fail.action_operation_fail.type_connection")).to_string(),
        true,
    )
}
