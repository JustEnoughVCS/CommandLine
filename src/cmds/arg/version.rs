#[derive(clap::Parser)]
pub struct JVVersionArgument {
    #[arg(short = 'c', long = "with-compile-info")]
    pub with_compile_info: bool,

    #[arg(long)]
    pub no_banner: bool,
}
