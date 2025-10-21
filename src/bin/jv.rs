use clap::{Parser, Subcommand, arg, command};
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
        },
        JustEnoughVcsWorkspaceCommand::Create(create_workspace_args) => {
            if create_workspace_args.help {
                println!("{}", md(t!("jv.create")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Init(init_workspace_args) => {
            if init_workspace_args.help {
                println!("{}", md(t!("jv.init")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Here(here_args) => {
            if here_args.help {
                println!("{}", md(t!("jv.here")));
                return;
            }
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
        }
        JustEnoughVcsWorkspaceCommand::Hold(hold_file_args) => {
            if hold_file_args.help {
                println!("{}", md(t!("jv.hold")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Throw(throw_file_args) => {
            if throw_file_args.help {
                println!("{}", md(t!("jv.throw")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Move(move_file_args) => {
            if move_file_args.help {
                println!("{}", md(t!("jv.move")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Export(export_file_args) => {
            if export_file_args.help {
                println!("{}", md(t!("jv.export")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Import(import_file_args) => {
            if import_file_args.help {
                println!("{}", md(t!("jv.import")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Direct(direct_args) => {
            if direct_args.help {
                println!("{}", md(t!("jv.direct")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Unstain(unstain_args) => {
            if unstain_args.help {
                println!("{}", md(t!("jv.unstain")));
                return;
            }
        }
        JustEnoughVcsWorkspaceCommand::Docs(docs_args) => {
            if docs_args.help {
                println!("{}", md(t!("jv.docs")));
                return;
            }
        }
    }
}
