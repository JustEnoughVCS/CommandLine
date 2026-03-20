#[derive(clap::Parser)]
pub struct JVWorkspaceAliasArgument {
    pub id: u32,

    #[arg(long = "to")]
    pub to: Option<u32>,

    #[arg(short = 'i', long = "insert")]
    pub insert: bool,

    #[arg(short = 'Q', long = "query")]
    pub query: bool,

    #[arg(short = 'e', long = "erase")]
    pub erase: bool,
}
