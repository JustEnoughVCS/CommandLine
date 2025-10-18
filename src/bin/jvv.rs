use std::env::set_current_dir;

use clap::{Parser, Subcommand};
use just_enough_vcs::{
    utils::{
        cfg_file::config::ConfigFile,
        string_proc::{self, pascal_case},
    },
    vcs::{
        connection::action_service::server_entry,
        constants::SERVER_FILE_VAULT,
        current::current_vault_path,
        data::{
            member::Member,
            vault::{Vault, config::VaultConfig},
        },
    },
};
use just_enough_vcs_cli::utils::{
    build_env_logger::build_env_logger, lang_selector::current_locales, md_colored::md,
};
use log::info;
use rust_i18n::{set_locale, t};
use tokio::fs::{self};

// Import i18n files
rust_i18n::i18n!("locales/help_docs", fallback = "en");

#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]
struct JustEnoughVcsVault {
    #[command(subcommand)]
    command: JustEnoughVcsCommand,
}

#[derive(Subcommand, Debug)]
enum JustEnoughVcsCommand {
    /// Get vault info in the current directory
    Here(HereArgs),

    /// Create a new directory and initialize a vault
    Create(CreateVaultArgs),

    /// Create a vault in the current directory
    Init(InitVaultArgs),

    /// Member manage
    #[command(subcommand)]
    Member(MemberManage),

    /// Manage service
    #[command(subcommand)]
    Service(ServiceManage),
}

#[derive(Subcommand, Debug)]
enum MemberManage {
    /// Register a member to the vault
    Register(MemberRegisterArgs),

    /// Remove a member from the vault
    Remove(MemberRemoveArgs),

    /// List all members in the vault
    List(MemberListArgs),

    /// Show help information
    #[command(alias = "--help", alias = "-h")]
    Help,
}

#[derive(Subcommand, Debug)]
enum ServiceManage {
    /// Listen connection at current vault
    Listen(ListenArgs),

    /// Show help information
    #[command(alias = "--help", alias = "-h")]
    Help,
}

#[derive(Parser, Debug)]
struct HereArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct CreateVaultArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Vault name to create
    vault_name: String,
}

#[derive(Parser, Debug)]
struct InitVaultArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct MemberRegisterArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Member name
    member_name: String,
}

#[derive(Parser, Debug)]
struct MemberRemoveArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Member name
    member_name: String,
}

#[derive(Parser, Debug)]
struct MemberListArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,
}

#[derive(Parser, Debug)]
struct ListenArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Disable logging
    #[arg(short, long)]
    no_log: bool,
}

#[tokio::main]
async fn main() {
    // Init i18n
    set_locale(&current_locales());

    // Init colored
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let Ok(parser) = JustEnoughVcsVault::try_parse() else {
        println!("{}", md(t!("jvv.help")));
        return;
    };

    match parser.command {
        JustEnoughVcsCommand::Here(here_args) => {
            if here_args.help {
                println!("{}", md(t!("jvv.here")));
                return;
            }
            jvv_here(here_args).await;
        }
        JustEnoughVcsCommand::Create(create_vault_args) => {
            if create_vault_args.help {
                println!("{}", md(t!("jvv.create")));
                return;
            }
            jvv_create(create_vault_args).await;
        }
        JustEnoughVcsCommand::Init(init_vault_args) => {
            if init_vault_args.help {
                println!("{}", md(t!("jvv.init")));
                return;
            }
            jvv_init(init_vault_args).await;
        }
        JustEnoughVcsCommand::Member(member_manage) => {
            let vault_cfg = VaultConfig::read()
                .await
                .unwrap_or_else(|_| panic!("{}", t!("jvv.fail.no_vault_here").trim().to_string()));

            let vault = match Vault::init_current_dir(vault_cfg) {
                Some(vault) => vault,
                None => {
                    eprintln!(
                        "{}",
                        t!("jvv.fail.jvcs", err = "Failed to initialize vault")
                    );
                    return;
                }
            };

            match member_manage {
                MemberManage::Register(member_register_args) => {
                    if member_register_args.help {
                        println!("{}", md(t!("jvv.member")));
                        return;
                    }
                    jvv_member_register(vault, member_register_args).await;
                }
                MemberManage::Remove(member_remove_args) => {
                    if member_remove_args.help {
                        println!("{}", md(t!("jvv.member")));
                        return;
                    }
                    jvv_member_remove(vault, member_remove_args).await;
                }
                MemberManage::List(member_list_args) => {
                    if member_list_args.help {
                        println!("{}", md(t!("jvv.member")));
                        return;
                    }
                    jvv_member_list(vault, member_list_args).await;
                }
                MemberManage::Help => {
                    println!("{}", md(t!("jvv.member")));
                    return;
                }
            }
        }
        JustEnoughVcsCommand::Service(service_manage) => match service_manage {
            ServiceManage::Listen(listen_args) => {
                if listen_args.help {
                    println!("{}", md(t!("jvv.service")));
                    return;
                }
                jvv_service_listen(listen_args).await;
            }
            ServiceManage::Help => {
                println!("{}", md(t!("jvv.service")));
                return;
            }
        },
    }
}

async fn jvv_here(_args: HereArgs) {
    let Some(current_vault) = current_vault_path() else {
        eprintln!("{}", t!("jvv.fail.here.cfg_not_found").trim());
        return;
    };

    // Read vault cfg
    let vault_cfg_file = current_vault.join(SERVER_FILE_VAULT);
    if !vault_cfg_file.exists() {
        eprintln!("{}", t!("jvv.fail.here.cfg_not_found").trim());
        return;
    }

    let vault_cfg = VaultConfig::read()
        .await
        .unwrap_or_else(|_| panic!("{}", t!("jvv.fail.here.cfg_not_found").trim().to_string()));

    // Get vault name
    let vault_name = vault_cfg.vault_name();

    // Success
    println!("{}", md(t!("jvv.success.here.root", name = vault_name)))
}

async fn jvv_init(_args: InitVaultArgs) {
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| panic!("{}", t!("jvv.fail.std.current_dir").trim().to_string()));
    if current_dir.read_dir().unwrap().next().is_some() {
        eprintln!("{}", t!("jvv.fail.init.not_empty"));
        return;
    }

    // Setup vault
    let vault_name = current_dir
        .file_name()
        .unwrap_or_else(|| panic!("{}", t!("jvv.fail.std.current_dir_name").trim().to_string()))
        .to_string_lossy()
        .to_string();
    let vault_name = pascal_case!(vault_name);

    if let Err(err) = Vault::setup_vault(current_dir.clone()).await {
        eprintln!("{}", t!("jvv.fail.jvcs", err = err.to_string()));
        return;
    }

    // Read vault cfg
    let mut vault_cfg = match VaultConfig::read().await {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{}", t!("jvv.fail.jvcs", err = err.to_string()));
            return;
        }
    };

    vault_cfg.change_name(vault_name);
    if let Err(err) = VaultConfig::write(&vault_cfg).await {
        eprintln!("{}", t!("jvv.fail.jvcs", err = err.to_string()));
        return;
    }

    // Success
    println!(
        "{}",
        t!("jvv.success.init", name = current_dir.to_string_lossy())
    )
}

async fn jvv_create(args: CreateVaultArgs) {
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| panic!("{}", t!("jvv.fail.std.current_dir").trim().to_string()));
    let target_dir = current_dir.join(args.vault_name.clone());

    // Create directory
    if fs::create_dir_all(&target_dir).await.is_err() {
        eprintln!(
            "{}",
            t!(
                "jvv.fail.tokio.fs.create_dir",
                dir = target_dir.to_string_lossy()
            )
        );
        return;
    }

    if target_dir.read_dir().unwrap().next().is_some() {
        eprintln!("{}", t!("jvv.fail.create.not_empty"));
        return;
    }

    // Setup vault
    let vault_name = pascal_case!(args.vault_name);
    if let Err(err) = Vault::setup_vault(target_dir.clone()).await {
        eprintln!("{}", t!("jvv.fail.jvcs", err = err.to_string()));
        return;
    }

    // Enter target_dir
    if set_current_dir(&target_dir).is_err() {
        eprintln!(
            "{}",
            t!(
                "jvv.fail.std.set_current_dir",
                dir = target_dir.to_string_lossy()
            )
        );
        return;
    }

    // Read vault cfg
    let mut vault_cfg = match VaultConfig::read().await {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{}", t!("jvv.fail.jvcs", err = err.to_string()));
            return;
        }
    };

    vault_cfg.change_name(vault_name);
    if let Err(err) = VaultConfig::write(&vault_cfg).await {
        eprintln!("{}", t!("jvv.fail.jvcs", err = err.to_string()));
        return;
    }

    // Success
    println!(
        "{}",
        t!("jvv.success.create", name = target_dir.to_string_lossy())
    )
}

async fn jvv_member_register(vault: Vault, args: MemberRegisterArgs) {
    let register = vault
        .register_member_to_vault(Member::new(args.member_name.clone()))
        .await;

    if register.is_err() {
        eprintln!("{}", t!("jvv.fail.member.register").trim());
        return;
    }

    println!(
        "{}",
        t!("jvv.success.member.register", member = args.member_name)
    )
}

async fn jvv_member_remove(vault: Vault, args: MemberRemoveArgs) {
    let _ = vault.remove_member_from_vault(&args.member_name);

    println!(
        "{}",
        t!("jvv.success.member.remove", member = args.member_name)
    )
}

async fn jvv_member_list(vault: Vault, _args: MemberListArgs) {
    // Get id list
    let ids = vault
        .member_ids()
        .unwrap_or_else(|_| panic!("{}", t!("jvv.fail.member.list").trim().to_string()));

    // Print header
    println!("{}", md(t!("jvv.success.member.list.header")));

    // Print list
    let mut i = 0;
    for member in ids {
        println!("{}. {}", i + 1, member);
        i += 1;
    }

    // Print footer
    println!("{}", md(t!("jvv.success.member.list.footer", num = i)));
}

async fn jvv_service_listen(args: ListenArgs) {
    let Some(current_vault) = current_vault_path() else {
        eprintln!("{}", t!("jvv.fail.here.cfg_not_found").trim());
        return;
    };

    if !args.no_log {
        let logs_dir = current_vault.join("logs");
        if let Err(_) = fs::create_dir_all(&logs_dir).await {
            eprintln!(
                "{}",
                t!(
                    "jvv.fail.tokio.fs.create_dir",
                    dir = logs_dir.to_string_lossy()
                )
            );
            return;
        }
        let now = chrono::Local::now();
        let log_filename = format!("log_{}.txt", now.format("%Y.%m.%d-%H:%M:%S"));
        build_env_logger(logs_dir.join(log_filename));
        info!(
            "{}",
            t!(
                "jvv.success.service.listen",
                path = current_vault.file_name().unwrap().display()
            )
        )
    }

    let _ = server_entry(current_vault).await;
}
