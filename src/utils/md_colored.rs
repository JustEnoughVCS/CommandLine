use colored::*;
use regex::Regex;

pub fn md(text: impl AsRef<str>) -> String {
    let bold_re = Regex::new(r"\*\*(.*?)\*\*").unwrap();
    let mut result = bold_re
        .replace_all(text.as_ref().trim(), |caps: &regex::Captures| {
            format!("{}", caps[1].bold())
        })
        .to_string();

    let italic_re = Regex::new(r"\*(.*?)\*").unwrap();
    result = italic_re
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}", caps[1].italic())
        })
        .to_string();

    result
}
