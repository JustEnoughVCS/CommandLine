use colored::*;
use regex::Regex;

pub struct SimpleTable {
    items: Vec<String>,
    line: Vec<Vec<String>>,
    length: Vec<usize>,
    padding: usize,
}

impl SimpleTable {
    /// Create a new Table
    pub fn new(items: Vec<impl Into<String>>) -> Self {
        Self::new_with_padding(items, 2)
    }

    /// Create a new Table with padding
    pub fn new_with_padding(items: Vec<impl Into<String>>, padding: usize) -> Self {
        let items: Vec<String> = items.into_iter().map(|v| v.into()).collect();
        let mut length = Vec::with_capacity(items.len());

        for item in &items {
            length.push(display_width(item));
        }

        SimpleTable {
            items,
            padding,
            line: Vec::new(),
            length,
        }
    }

    /// Push a new row of items to the table
    pub fn push_item(&mut self, items: Vec<impl Into<String>>) {
        let items: Vec<String> = items.into_iter().map(|v| v.into()).collect();

        let mut processed_items = Vec::with_capacity(self.items.len());

        for i in 0..self.items.len() {
            if i < items.len() {
                processed_items.push(items[i].clone());
            } else {
                processed_items.push(String::new());
            }
        }

        for (i, d) in processed_items.iter().enumerate() {
            let d_len = display_width(d);
            if d_len > self.length[i] {
                self.length[i] = d_len;
            }
        }

        self.line.push(processed_items);
    }

    /// Get the current maximum column widths
    fn get_column_widths(&self) -> &[usize] {
        &self.length
    }
}

impl std::fmt::Display for SimpleTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let column_widths = self.get_column_widths();

        // Build the header row
        let header: Vec<String> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let target_width = column_widths[i] + self.padding;
                let current_width = display_width(item);
                let space_count = target_width - current_width;
                let space = " ".repeat(space_count);
                let result = format!("{}{}", item, space);
                result
            })
            .collect();
        writeln!(f, "{}", header.join(""))?;

        // Build each data row
        for row in &self.line {
            let formatted_row: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let target_width = column_widths[i] + self.padding;
                    let current_width = display_width(cell);
                    let space_count = target_width - current_width;
                    let spaces = " ".repeat(space_count);
                    let result = format!("{}{}", cell, spaces);
                    result
                })
                .collect();
            writeln!(f, "{}", formatted_row.join(""))?;
        }

        Ok(())
    }
}

fn display_width(s: &str) -> usize {
    // Filter out ANSI escape sequences before calculating width
    let filtered_bytes = strip_ansi_escapes::strip(s);
    let filtered_str = match std::str::from_utf8(&filtered_bytes) {
        Ok(s) => s,
        Err(_) => s, // Fallback to original string if UTF-8 conversion fails
    };

    let mut width = 0;
    for c in filtered_str.chars() {
        if c.is_ascii() {
            width += 1;
        } else {
            width += 2;
        }
    }
    width
}

/// Convert byte size to a human-readable string format
///
/// Automatically selects the appropriate unit (B, KB, MB, GB, TB) based on the byte size
/// and formats it as a string with two decimal places
pub fn size_str(total_size: usize) -> String {
    if total_size < 1024 {
        format!("{} B", total_size)
    } else if total_size < 1024 * 1024 {
        format!("{:.2} KB", total_size as f64 / 1024.0)
    } else if total_size < 1024 * 1024 * 1024 {
        format!("{:.2} MB", total_size as f64 / (1024.0 * 1024.0))
    } else if total_size < 1024 * 1024 * 1024 * 1024 {
        format!("{:.2} GB", total_size as f64 / (1024.0 * 1024.0 * 1024.0))
    } else {
        format!(
            "{:.2} TB",
            total_size as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)
        )
    }
}

// Convert the Markdown formatted text into a format supported by the command line
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
