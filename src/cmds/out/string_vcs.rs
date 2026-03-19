use serde::Serialize;

#[derive(Serialize)]
pub struct JVStringVecOutput {
    pub vec: Vec<String>,
}

impl From<Vec<String>> for JVStringVecOutput {
    fn from(vec: Vec<String>) -> Self {
        JVStringVecOutput { vec }
    }
}

impl From<JVStringVecOutput> for Vec<String> {
    fn from(jv_string_vec: JVStringVecOutput) -> Self {
        jv_string_vec.vec
    }
}

impl AsRef<Vec<String>> for JVStringVecOutput {
    fn as_ref(&self) -> &Vec<String> {
        &self.vec
    }
}

impl AsRef<[String]> for JVStringVecOutput {
    fn as_ref(&self) -> &[String] {
        &self.vec
    }
}

impl std::ops::Deref for JVStringVecOutput {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl IntoIterator for JVStringVecOutput {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a> IntoIterator for &'a JVStringVecOutput {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl<'a> IntoIterator for &'a mut JVStringVecOutput {
    type Item = &'a mut String;
    type IntoIter = std::slice::IterMut<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter_mut()
    }
}
