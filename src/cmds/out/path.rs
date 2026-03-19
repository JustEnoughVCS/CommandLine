use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct JVPathOutput {
    pub path: PathBuf,
}

impl From<PathBuf> for JVPathOutput {
    fn from(path: PathBuf) -> Self {
        JVPathOutput { path }
    }
}

impl From<JVPathOutput> for PathBuf {
    fn from(jv_path: JVPathOutput) -> Self {
        jv_path.path
    }
}

impl AsRef<PathBuf> for JVPathOutput {
    fn as_ref(&self) -> &PathBuf {
        &self.path
    }
}

impl AsRef<std::path::Path> for JVPathOutput {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}

impl std::ops::Deref for JVPathOutput {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}
