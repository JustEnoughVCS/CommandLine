#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]

struct JustEnoughVcsInputer {
    #[command(subcommand)]
    command: JustEnoughVcsInputerCommand,
}

#[derive(Subcommand, Debug)]
enum JustEnoughVcsInputerCommand {
    /// Version information
    #[command(alias = "--version", alias = "-v")]
    Version(VersionArgs),
}
