use std::{net::SocketAddr, str::FromStr};

use clap::{Parser, Subcommand, arg, command};
use just_enough_vcs::{
    system::action_system::action::ActionContext,
    utils::tcp_connection::error::TcpTargetError,
    vcs::{actions::local_actions::proc_set_upstream_vault_action, registry::client_registry},
};
use just_enough_vcs_cli::utils::{lang_selector::current_locales, md_colored::md};
use rust_i18n::{set_locale, t};

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
    #[command(subcommand)]
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
    Remove(AccountRemoveArgs),
}

#[derive(Subcommand, Debug)]
enum SheetManage {
    /// Show help information
    #[command(alias = "--help", alias = "-h")]
    Help,
}

#[derive(Parser, Debug)]
struct AccountAddArgs {
    /// Member name
    member_name: String,
}

#[derive(Parser, Debug)]
struct AccountRemoveArgs {
    /// Member name
    member_name: String,
}

#[derive(Parser, Debug)]
struct CreateWorkspaceArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct InitWorkspaceArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct HereArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
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
        JustEnoughVcsWorkspaceCommand::Account(account_manage) => match account_manage {
            AccountManage::Help => {
                println!("{}", md(t!("jv.account")));
            }
            AccountManage::Add(account_add_args) => todo!(),
            AccountManage::Remove(account_remove_args) => todo!(),
        },
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

async fn jv_create(_args: CreateWorkspaceArgs) {
    todo!()
}

async fn jv_init(_args: InitWorkspaceArgs) {
    todo!()
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

async fn jv_direct(args: DirectArgs) {
    let pool = client_registry::client_action_pool();
    let upstream = match SocketAddr::from_str(&args.upstream) {
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
