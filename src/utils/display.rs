use colored::*;
use std::collections::VecDeque;

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

    /// Insert a new row of items at the specified index
    pub fn insert_item(&mut self, index: usize, items: Vec<impl Into<String>>) {
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

        self.line.insert(index, processed_items);
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

pub fn display_width(s: &str) -> usize {
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
    let text = text.as_ref().trim();
    let mut result = String::new();
    let mut color_stack: VecDeque<String> = VecDeque::new();

    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();

    while i < chars.len() {
        // Check for escape character \
        if chars[i] == '\\' && i + 1 < chars.len() {
            let escaped_char = chars[i + 1];
            // Only escape specific characters
            if matches!(escaped_char, '*' | '<' | '>' | '`') {
                let mut escaped_text = escaped_char.to_string();

                // Apply current color stack
                for color in color_stack.iter().rev() {
                    escaped_text = apply_color(&escaped_text, color);
                }

                result.push_str(&escaped_text);
                i += 2;
                continue;
            }
        }

        // Check for color tag start [[color]]
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            let mut j = i + 2;
            while j < chars.len()
                && !(chars[j] == ']' && j + 1 < chars.len() && chars[j + 1] == ']')
            {
                j += 1;
            }

            if j + 1 < chars.len() {
                let tag_content: String = chars[i + 2..j].iter().collect();

                // Check if it's a closing tag [[/]]
                if tag_content == "/" {
                    color_stack.pop_back();
                    i = j + 2;
                    continue;
                }

                // It's a color tag
                color_stack.push_back(tag_content.clone());
                i = j + 2;
                continue;
            }
        }

        // Check for bold **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            let mut j = i + 2;
            while j + 1 < chars.len() && !(chars[j] == '*' && chars[j + 1] == '*') {
                j += 1;
            }

            if j + 1 < chars.len() {
                let bold_text: String = chars[i + 2..j].iter().collect();
                let mut formatted_text = bold_text.bold().to_string();

                // Apply current color stack
                for color in color_stack.iter().rev() {
                    formatted_text = apply_color(&formatted_text, color);
                }

                result.push_str(&formatted_text);
                i = j + 2;
                continue;
            }
        }

        // Check for italic *text*
        if chars[i] == '*' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '*' {
                j += 1;
            }

            if j < chars.len() {
                let italic_text: String = chars[i + 1..j].iter().collect();
                let mut formatted_text = italic_text.italic().to_string();

                // Apply current color stack
                for color in color_stack.iter().rev() {
                    formatted_text = apply_color(&formatted_text, color);
                }

                result.push_str(&formatted_text);
                i = j + 1;
                continue;
            }
        }

        // Check for inline code `text`
        if chars[i] == '`' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '`' {
                j += 1;
            }

            if j < chars.len() {
                // Include the backticks in the output
                let code_text: String = chars[i..=j].iter().collect();
                let mut formatted_text = code_text.green().to_string();

                // Apply current color stack
                for color in color_stack.iter().rev() {
                    formatted_text = apply_color(&formatted_text, color);
                }

                result.push_str(&formatted_text);
                i = j + 1;
                continue;
            }
        }

        // Check for angle-bracketed content <text>
        if chars[i] == '<' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '>' {
                j += 1;
            }

            if j < chars.len() {
                // Include the angle brackets in the output
                let angle_text: String = chars[i..=j].iter().collect();
                let mut formatted_text = angle_text.cyan().to_string();

                // Apply current color stack
                for color in color_stack.iter().rev() {
                    formatted_text = apply_color(&formatted_text, color);
                }

                result.push_str(&formatted_text);
                i = j + 1;
                continue;
            }
        }

        // Regular character
        let mut current_char = chars[i].to_string();

        // Apply current color stack
        for color in color_stack.iter().rev() {
            current_char = apply_color(&current_char, color);
        }

        result.push_str(&current_char);
        i += 1;
    }

    result
}

// Helper function to apply color to text
fn apply_color(text: &str, color_name: &str) -> String {
    match color_name {
        // Normal colors
        "black" => text.black().to_string(),
        "red" => text.red().to_string(),
        "green" => text.green().to_string(),
        "yellow" => text.yellow().to_string(),
        "blue" => text.blue().to_string(),
        "magenta" => text.magenta().to_string(),
        "cyan" => text.cyan().to_string(),
        "white" => text.white().to_string(),
        "bright_black" => text.bright_black().to_string(),
        "bright_red" => text.bright_red().to_string(),
        "bright_green" => text.bright_green().to_string(),
        "bright_yellow" => text.bright_yellow().to_string(),
        "bright_blue" => text.bright_blue().to_string(),
        "bright_magenta" => text.bright_magenta().to_string(),
        "bright_cyan" => text.bright_cyan().to_string(),
        "bright_white" => text.bright_white().to_string(),

        // Short aliases for bright colors
        "b_black" => text.bright_black().to_string(),
        "b_red" => text.bright_red().to_string(),
        "b_green" => text.bright_green().to_string(),
        "b_yellow" => text.bright_yellow().to_string(),
        "b_blue" => text.bright_blue().to_string(),
        "b_magenta" => text.bright_magenta().to_string(),
        "b_cyan" => text.bright_cyan().to_string(),
        "b_white" => text.bright_white().to_string(),

        // Gray colors using truecolor
        "gray" | "grey" => text.truecolor(128, 128, 128).to_string(),
        "bright_gray" | "bright_grey" => text.truecolor(192, 192, 192).to_string(),
        "b_gray" | "b_grey" => text.truecolor(192, 192, 192).to_string(),

        // Default to white if color not recognized
        _ => text.to_string(),
    }
}
