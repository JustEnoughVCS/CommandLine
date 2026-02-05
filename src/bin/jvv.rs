use clap::{Parser, Subcommand};
use cli_utils::{
    display::{md, size_str},
    env::current_locales,
    logger::build_env_logger,
};
use just_enough_vcs::{
    data::compile_info::CoreCompileInfo,
    lib::{
        connection::action_service::server_entry,
        constants::SERVER_FILE_VAULT,
        data::{
            member::Member,
            vault::{Vault, vault_config::VaultConfig},
        },
        env::current_vault_path,
    },
    utils::{
        cfg_file::config::ConfigFile,
        string_proc::{self, pascal_case},
    },
};
use just_enough_vcs_cli::data::compile_info::CompileInfo;
use log::{error, info};
use rust_i18n::{set_locale, t};
use tokio::fs::{self};

// Import i18n files
rust_i18n::i18n!("resources/locales", fallback = "en");

#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]
struct JustEnoughVcsVault {
    #[command(subcommand)]
    command: JustEnoughVcsVaultCommand,
}

#[derive(Subcommand, Debug)]
enum JustEnoughVcsVaultCommand {
    /// Version information
    #[command(alias = "--version", alias = "-v")]
    Version(VersionArgs),

    /// Get vault info in the current directory
    #[command(alias = "-H")]
    Here(HereArgs),

    /// Create a new directory and initialize a vault
    #[command(alias = "-c")]
    Create(CreateVaultArgs),

    /// Create a vault in the current directory
    #[command(alias = "-i")]
    Init(InitVaultArgs),

    /// Member manage
    #[command(subcommand, alias = "-m")]
    Member(MemberManage),

    /// Manage service
    #[command(subcommand)]
    Service(ServiceManage),

    // Short commands
    #[command(alias = "-l", alias = "listen")]
    ServiceListen(ListenArgs),

    // List all members
    #[command(alias = "-M", alias = "members")]
    MemberList,
}

#[derive(Parser, Debug)]
struct VersionArgs {
    #[arg(short = 'C', long = "compile-info")]
    compile_info: bool,
}

#[derive(Subcommand, Debug)]
enum MemberManage {
    /// Register a member to the vault
    #[command(alias = "+")]
    Register(MemberRegisterArgs),

    /// Remove a member from the vault
    #[command(alias = "-")]
    Remove(MemberRemoveArgs),

    /// List all members in the vault
    #[command(alias = "ls")]
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

    /// Show raw output for list
    #[arg(short, long)]
    raw: bool,
}

#[derive(Parser, Debug)]
struct ListenArgs {
    /// Show help information
    #[arg(short, long)]
    help: bool,

    /// Disable logging (Override profile)
    #[arg(short, long)]
    no_log: bool,

    /// Show logging (Override profile)
    #[arg(short, long)]
    show_log: bool,

    /// Custom port
    #[arg(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    // Init i18n
    set_locale(&current_locales());

    // Init colored
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let Ok(parser) = JustEnoughVcsVault::try_parse() else {
        println!("{}", md(t!("jvv.help")));
        return;
    };

    match parser.command {
        JustEnoughVcsVaultCommand::Version(version_args) => {
            let compile_info = CompileInfo::default();
            let core_compile_info = CoreCompileInfo::default();
            println!(
                "{}",
                md(t!(
                    "jvv.version.header",
                    version = compile_info.cli_version,
                    vcs_version = core_compile_info.vcs_version
                ))
            );

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

        JustEnoughVcsVaultCommand::Here(here_args) => {
            if here_args.help {
                println!("{}", md(t!("jvv.here")));
                return;
            }
            jvv_here(here_args).await;
        }
        JustEnoughVcsVaultCommand::Create(create_vault_args) => {
            if create_vault_args.help {
                println!("{}", md(t!("jvv.create")));
                return;
            }
            jvv_create(create_vault_args).await;
        }
        JustEnoughVcsVaultCommand::Init(init_vault_args) => {
            if init_vault_args.help {
                println!("{}", md(t!("jvv.init")));
                return;
            }
            jvv_init(init_vault_args).await;
        }
        JustEnoughVcsVaultCommand::Member(member_manage) => {
            let vault_cfg = match VaultConfig::read().await {
                Ok(cfg) => cfg,
                Err(_) => {
                    eprintln!("{}", t!("jvv.fail.no_vault_here").trim());
                    return;
                }
            };

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
        JustEnoughVcsVaultCommand::Service(service_manage) => match service_manage {
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
        // Short commands
        JustEnoughVcsVaultCommand::ServiceListen(listen_args) => {
            if listen_args.help {
                println!("{}", md(t!("jvv.service")));
                return;
            }
            jvv_service_listen(listen_args).await;
        }
        JustEnoughVcsVaultCommand::MemberList => {
            let vault_cfg = match VaultConfig::read().await {
                Ok(cfg) => cfg,
                Err(_) => {
                    eprintln!("{}", t!("jvv.fail.no_vault_here").trim());
                    return;
                }
            };

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
            jvv_member_list(
                vault,
                MemberListArgs {
                    help: false,
                    raw: false,
                },
            )
            .await;
        }
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

    let vault_cfg = match VaultConfig::read().await {
        Ok(cfg) => cfg,
        Err(_) => {
            eprintln!("{}", t!("jvv.fail.here.cfg_not_found").trim());
            return;
        }
    };

    // Get vault name
    let vault_name = vault_cfg.vault_name().clone();

    // Get vault
    let Some(vault) = Vault::init(vault_cfg, current_vault) else {
        eprintln!("{}", t!("jvv.fail.here.vault_init_failed").trim());
        return;
    };

    // Get sheet count
    let num_sheets = vault.sheet_names().unwrap().iter().len();

    // Get virtual file count and total size recursively
    let virtual_file_root = vault.virtual_file_storage_dir();
    let mut num_vf = 0;
    let mut total_size = 0;

    // Recursive function to calculate total size and count files with specific name
    async fn calculate_total_size_and_vf_count(
        path: std::path::PathBuf,
        file_count: &mut u64,
        total_size: &mut u64,
    ) -> std::io::Result<()> {
        let mut entries = fs::read_dir(&path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                *total_size += metadata.len();
                // Check if file name matches SERVER_NAME_VF_META
                if entry.file_name() == just_enough_vcs::lib::constants::SERVER_NAME_VF_META {
                    *file_count += 1;
                }
            } else if metadata.is_dir() {
                Box::pin(calculate_total_size_and_vf_count(
                    entry.path(),
                    file_count,
                    total_size,
                ))
                .await?;
            }
        }
        Ok(())
    }

    // Calculate with timeout
    let timeout_duration = std::time::Duration::from_millis(1200);
    let size_result = tokio::time::timeout(timeout_duration, async {
        calculate_total_size_and_vf_count(virtual_file_root.clone(), &mut num_vf, &mut total_size)
            .await
    })
    .await;

    match size_result {
        Ok(Ok(_)) => {
            // Calculation completed within timeout
        }
        Ok(Err(e)) => {
            eprintln!(
                "{}",
                t!("jvv.fail.here.size_calc_error", error = e.to_string()).trim()
            );
            return;
        }
        Err(_) => {
            // Timeout occurred
            println!("{}", t!("jvv.info.here.analyzing_size").trim());
            // Continue with partial calculation (num_vf and total_size as calculated so far)
        }
    }

    // Get member count
    let num_mem = match vault.member_ids() {
        Ok(ids) => ids.len(),
        Err(_) => {
            eprintln!("{}", t!("jvv.fail.here.member_ids_failed").trim());
            return;
        }
    };

    // Get public key count
    let mut num_pk = 0;
    let member_ids = match vault.member_ids() {
        Ok(ids) => ids,
        Err(_) => {
            eprintln!("{}", t!("jvv.fail.here.member_ids_failed").trim());
            return;
        }
    };

    for member_id in member_ids {
        if vault.member_key_path(&member_id).exists() {
            num_pk += 1;
        }
    }

    // Get reference sheet info
    let ref_sheet = match vault.sheet(&"ref".to_string()).await {
        Ok(sheet) => sheet,
        Err(_) => {
            eprintln!("{}", t!("jvv.fail.here.ref_sheet_not_found").trim());
            return;
        }
    };
    let num_ref_sheet_managed_files = ref_sheet.mapping().len();

    // Success
    println!(
        "{}",
        md(t!(
            "jvv.success.here.info",
            name = vault_name,
            num_sheets = num_sheets,
            num_vf = num_vf,
            num_mem = num_mem,
            num_pk = num_pk,
            num_ref_sheet_managed_files = num_ref_sheet_managed_files,
            total_size = size_str(total_size as usize)
        ))
    )
}

async fn jvv_init(_args: InitVaultArgs) {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("{}", t!("jvv.fail.std.current_dir").trim());
            return;
        }
    };
    if let Ok(mut entries) = current_dir.read_dir()
        && entries.next().is_some()
    {
        eprintln!("{}", t!("jvv.fail.init.not_empty"));
        return;
    }

    // Setup vault
    let vault_name = match current_dir.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            eprintln!("{}", t!("jvv.fail.std.current_dir_name").trim());
            return;
        }
    };
    let vault_name = pascal_case!(vault_name);

    if let Err(err) = Vault::setup_vault(current_dir.clone(), vault_name).await {
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
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("{}", t!("jvv.fail.std.current_dir").trim());
            return;
        }
    };
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

    if let Ok(mut entries) = target_dir.read_dir()
        && entries.next().is_some()
    {
        eprintln!("{}", t!("jvv.fail.create.not_empty"));
        return;
    }

    // Setup vault
    let vault_name = pascal_case!(args.vault_name);
    if let Err(err) = Vault::setup_vault(target_dir.clone(), vault_name).await {
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

async fn jvv_member_list(vault: Vault, args: MemberListArgs) {
    // Get id list
    let ids = match vault.member_ids() {
        Ok(ids) => ids,
        Err(_) => {
            if !args.raw {
                eprintln!("{}", t!("jvv.fail.member.list").trim());
            }
            return;
        }
    };

    let mut members: Vec<String> = ids.into_iter().collect();

    // Sort members to put "host" first if it exists
    members.sort_by(|a, b| {
        if a == "host" {
            std::cmp::Ordering::Less
        } else if b == "host" {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });

    if args.raw {
        for member in members {
            println!("{}", member);
        }
    } else {
        // Print header
        println!(
            "{}",
            md(t!("jvv.success.member.list.header", num = members.len()))
        );

        // Print list
        let mut i = 0;
        let mut has_pubkey = 0;
        for member in members {
            println!("{}. {} {}", i + 1, &member, {
                // Key registered
                if vault.member_key_path(&member).exists() {
                    has_pubkey += 1;
                    t!("jvv.success.member.list.status_key_registered")
                } else {
                    std::borrow::Cow::Borrowed("")
                }
            });
            i += 1;
        }

        println!(
            "{}",
            md(t!("jvv.success.member.list.footer", num = has_pubkey))
        );
    }
}

async fn jvv_service_listen(args: ListenArgs) {
    let Some(current_vault) = current_vault_path() else {
        eprintln!("{}", t!("jvv.fail.here.cfg_not_found").trim());
        return;
    };

    let Ok(vault_cfg) = VaultConfig::read().await else {
        eprintln!("{}", t!("jvv.fail.here.cfg_not_found").trim());
        return;
    };

    let show_logger = if args.no_log && !args.show_log {
        false
    } else if !args.no_log && args.show_log {
        true
    } else if !args.no_log && !args.show_log {
        // Read profile
        vault_cfg.server_config().is_logger_enabled()
    } else {
        eprintln!("{}", md(t!("jvv.fail.service.wtf_show_log_and_no_log")));
        return;
    };

    if show_logger {
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
        let log_filename = format!("log_{}.txt", now.format("%Y-%m-%d-%H-%M-%S"));
        build_env_logger(
            logs_dir.join(log_filename),
            vault_cfg.server_config().logger_level(),
        );
        info!(
            "{}",
            t!(
                "jvv.success.service.listen_start",
                path = match current_vault.file_name() {
                    Some(name) => name.to_string_lossy(),
                    None => std::borrow::Cow::Borrowed("unknown"),
                }
            )
        )
    }

    let port = args.port.unwrap_or_default();
    match server_entry(current_vault, port).await {
        Ok(_) => {
            info!("{}", t!("jvv.success.service.listen_done").trim());
        }
        Err(e) => {
            error!(
                "{}",
                t!("jvv.fail.service.listen_done", error = e.to_string()).trim()
            );
        }
    }
}
