pub struct JVWorkspaceAliasInput {
    pub query: bool,
    pub mode: JVWorkspaceAliasInputMode,
}

pub enum JVWorkspaceAliasInputMode {
    Insert(u32, u32),
    Erase(u32),
    None(u32),
}
