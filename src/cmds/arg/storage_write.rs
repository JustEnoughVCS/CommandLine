use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct JVStorageWriteArgument {
    pub file: PathBuf,
    pub storage: PathBuf,

    #[arg(short = 'o', long = "output")]
    pub output_index: Option<PathBuf>,

    #[arg(long = "line")]
    pub line_chunking: bool,

    #[arg(long = "cdc", default_value_t = 0)]
    pub cdc_chunking: u32,

    #[arg(long = "fixed", default_value_t = 0)]
    pub fixed_chunking: u32,

    #[arg(long)]
    pub b: bool,

    #[arg(long)]
    pub kb: bool, // default chunk size unit

    #[arg(long)]
    pub mb: bool,

    #[arg(long)]
    pub gb: bool,
}
