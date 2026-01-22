#[derive(thiserror::Error, Debug)]
pub enum CmdPrepareError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Error(String),
}

impl CmdPrepareError {
    pub fn new(msg: impl AsRef<str>) -> Self {
        CmdPrepareError::Error(msg.as_ref().to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CmdExecuteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Content not prepared, cannot run")]
    Prepare(#[from] CmdPrepareError),

    #[error("{0}")]
    Error(String),
}

impl CmdExecuteError {
    pub fn new(msg: impl AsRef<str>) -> Self {
        CmdExecuteError::Error(msg.as_ref().to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CmdRenderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Preparation failed, cannot render")]
    Prepare(#[from] CmdPrepareError),

    #[error("Execution failed, no output content obtained before rendering")]
    Execute(#[from] CmdExecuteError),

    #[error("{0}")]
    Error(String),
}

impl CmdRenderError {
    pub fn new(msg: impl AsRef<str>) -> Self {
        CmdRenderError::Error(msg.as_ref().to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CmdProcessError {
    #[error("Prepare error: {0}")]
    Prepare(#[from] CmdPrepareError),

    #[error("Execute error: {0}")]
    Execute(#[from] CmdExecuteError),

    #[error("Render error: {0}")]
    Render(#[from] CmdRenderError),

    #[error("{0}")]
    Error(String),

    #[error("Node `{0}` not found!")]
    NoNodeFound(String),

    #[error("No matching command found")]
    NoMatchingCommand,

    #[error("Ambiguous command, multiple matches found")]
    AmbiguousCommand(Vec<String>),

    #[error("Parse error")]
    ParseError(String),
}

impl CmdProcessError {
    pub fn new(msg: impl AsRef<str>) -> Self {
        CmdProcessError::Error(msg.as_ref().to_string())
    }

    pub fn prepare_err(&self) -> Option<&CmdPrepareError> {
        match self {
            CmdProcessError::Prepare(e) => Some(e),
            _ => None,
        }
    }

    pub fn execute_err(&self) -> Option<&CmdExecuteError> {
        match self {
            CmdProcessError::Execute(e) => Some(e),
            _ => None,
        }
    }

    pub fn render_err(&self) -> Option<&CmdRenderError> {
        match self {
            CmdProcessError::Render(e) => Some(e),
            _ => None,
        }
    }
}
