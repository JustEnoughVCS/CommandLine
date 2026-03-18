// Why not directly design new, delete, print_path as Option<String>?
// Because the former would only allow the following syntax:
//   jvn workspace sheet --new sheet
// But by separating name and operation, it can simultaneously support:
//   jvn workspace sheet sheet --new

#[derive(clap::Parser)]
pub struct JVWorkspaceSheetArgument {
    pub name: Option<String>,

    #[arg(short = 'n', long = "new")]
    pub new: bool,

    #[arg(short = 'd', long = "delete")]
    pub delete: bool,

    #[arg(short = 'A', long = "list-all")]
    pub list_all: bool,

    #[arg(short = 'p', long = "print-path")]
    pub print_path: bool,
}
