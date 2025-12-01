use std::path::PathBuf;

use crate::utils::globber::constants::{SPLIT_STR, get_base_dir_current};

pub struct Globber {
    pattern: String,
    base: PathBuf,
    names: Vec<String>,
}

#[allow(dead_code)]
impl Globber {
    pub fn new(pattern: String, base: PathBuf) -> Self {
        Self {
            pattern,
            base,
            names: Vec::new(),
        }
    }

    pub fn names(&self) -> Vec<&String> {
        self.names.iter().collect()
    }

    pub fn into_names(self) -> Vec<String> {
        self.names
    }

    pub fn paths(&self) -> Vec<PathBuf> {
        self.names.iter().map(|n| self.base.join(n)).collect()
    }

    pub fn glob<F>(mut self, get_names: F) -> Result<Self, std::io::Error>
    where
        F: Fn(PathBuf) -> Vec<GlobItem>,
    {
        let full_path = format!("{}{}{}", self.base.display(), SPLIT_STR, self.pattern);

        let (path, pattern) = if let Some(last_split) = full_path.rfind(SPLIT_STR) {
            let (path_part, pattern_part) = full_path.split_at(last_split);
            let mut path = path_part.to_string();
            if !path.ends_with(SPLIT_STR) {
                path.push_str(SPLIT_STR);
            }
            (path, pattern_part[SPLIT_STR.len()..].to_string())
        } else {
            (String::default(), full_path)
        };

        let pattern = if pattern.is_empty() {
            "*".to_string()
        } else if pattern.ends_with(SPLIT_STR) {
            format!("{}*", pattern)
        } else {
            pattern
        };

        if !pattern.contains('*') && !pattern.contains('?') {
            self.names = vec![pattern];
            return Ok(self);
        }

        let mut collected = Vec::new();

        collect_files(&path.into(), String::new(), &mut collected, &get_names);
        fn collect_files<F>(
            base: &PathBuf,
            current: String,
            file_names: &mut Vec<String>,
            get_names: &F,
        ) where
            F: Fn(PathBuf) -> Vec<GlobItem>,
        {
            let current_path = if current.is_empty() {
                base.clone()
            } else {
                base.join(&current)
            };

            let items = get_names(current_path);
            for item in items {
                match item {
                    GlobItem::File(file_name) => {
                        let relative_path = if current.is_empty() {
                            file_name
                        } else {
                            format!("{}{}{}", current, SPLIT_STR, file_name)
                        };
                        file_names.push(relative_path)
                    }
                    GlobItem::Directory(dir_name) => {
                        let new_current = if current.is_empty() {
                            dir_name
                        } else {
                            format!("{}{}{}", current, SPLIT_STR, dir_name)
                        };
                        collect_files(base, new_current, file_names, get_names);
                    }
                }
            }
        }

        self.names = collected
            .iter()
            .filter_map(|name| match_pattern(name, &pattern))
            .collect();

        Ok(self)
    }
}

fn match_pattern(name: &str, pattern: &str) -> Option<String> {
    if pattern.is_empty() {
        return None;
    }

    let name_chars: Vec<char> = name.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    let mut name_idx = 0;
    let mut pattern_idx = 0;
    let mut star_idx = -1;
    let mut match_idx = -1;

    while name_idx < name_chars.len() {
        if pattern_idx < pattern_chars.len()
            && (pattern_chars[pattern_idx] == '?'
                || pattern_chars[pattern_idx] == name_chars[name_idx])
        {
            name_idx += 1;
            pattern_idx += 1;
        } else if pattern_idx < pattern_chars.len() && pattern_chars[pattern_idx] == '*' {
            star_idx = pattern_idx as i32;
            match_idx = name_idx as i32;
            pattern_idx += 1;
        } else if star_idx != -1 {
            pattern_idx = (star_idx + 1) as usize;
            match_idx += 1;
            name_idx = match_idx as usize;
        } else {
            return None;
        }
    }

    while pattern_idx < pattern_chars.len() && pattern_chars[pattern_idx] == '*' {
        pattern_idx += 1;
    }

    if pattern_idx == pattern_chars.len() {
        Some(name.to_string())
    } else {
        None
    }
}

impl<T: AsRef<str>> From<T> for Globber {
    fn from(pattern: T) -> Self {
        let (base_dir, pattern) = get_base_dir_current(pattern.as_ref().to_string());
        Self::new(pattern, base_dir)
    }
}

pub enum GlobItem {
    File(String),
    Directory(String),
}

pub mod constants {
    use std::{env::current_dir, path::PathBuf};

    #[cfg(unix)]
    pub(crate) const CURRENT_DIR_PREFIX: &str = "./";
    #[cfg(windows)]
    pub(crate) const CURRENT_DIR_PREFIX: &str = ".\\";

    #[cfg(unix)]
    pub(crate) const USER_DIR_PREFIX: &str = "~";
    #[cfg(windows)]
    pub(crate) const USER_DIR_PREFIX: &str = "~\\";

    #[cfg(unix)]
    pub(crate) const ROOT_DIR_PREFIX: &str = "/";
    #[cfg(windows)]
    pub(crate) const ROOT_DIR_PREFIX: &str = "\\";

    #[cfg(unix)]
    pub(crate) const SPLIT_STR: &str = "/";
    #[cfg(windows)]
    pub(crate) const SPLIT_STR: &str = "\\";

    pub fn get_base_dir_current(input: String) -> (PathBuf, String) {
        get_base_dir(input, current_dir().unwrap_or_default())
    }

    pub fn get_base_dir(input: String, current_dir: PathBuf) -> (PathBuf, String) {
        if let Some(remaining) = input.strip_prefix(CURRENT_DIR_PREFIX) {
            (current_dir, remaining.to_string())
        } else if let Some(remaining) = input.strip_prefix(USER_DIR_PREFIX) {
            (dirs::home_dir().unwrap_or_default(), remaining.to_string())
        } else if let Some(remaining) = input.strip_prefix(ROOT_DIR_PREFIX) {
            {
                #[cfg(unix)]
                {
                    (PathBuf::from(ROOT_DIR_PREFIX), remaining.to_string())
                }
                #[cfg(windows)]
                {
                    let current_drive = current_dir()
                        .unwrap_or_default()
                        .components()
                        .find_map(|comp| {
                            if let std::path::Component::Prefix(prefix) = comp {
                                Some(prefix)
                            } else {
                                None
                            }
                        })
                        .and_then(|prefix| {
                            if let std::path::Prefix::Disk(drive_letter)
                            | std::path::Prefix::VerbatimDisk(drive_letter) = prefix
                            {
                                Some((drive_letter as char).to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "C".to_string());

                    (
                        PathBuf::from(format!("{}:{}", current_drive, ROOT_DIR_PREFIX)),
                        remaining.to_string(),
                    )
                }
            }
        } else {
            (current_dir, input)
        }
    }
}
