use std::collections::VecDeque;

use crossterm::style::Stylize;

/// Trait for adding markdown formatting to strings
pub trait Colorful {
    fn colorful(&self) -> String;
}

impl Colorful for &str {
    fn colorful(&self) -> String {
        colorful(self)
    }
}

impl Colorful for String {
    fn colorful(&self) -> String {
        colorful(self)
    }
}

/// Converts a string to colored/formatted text with ANSI escape codes.
///
/// Supported syntax:
/// - Bold: `**text**`
/// - Italic: `*text*`
/// - Underline: `_text_`
/// - Angle-bracketed content: `<text>` (displayed as cyan)
/// - Inline code: `` `text` `` (displayed as green)
/// - Color tags: `[[color_name]]` and `[[/]]` to close color
/// - Escape characters: `\*`, `\<`, `\>`, `` \` ``, `\_` for literal characters
///
/// Color tags support the following color names:
/// Color tags support the following color names:
///
/// | Type                  | Color Names                                                                 |
/// |-----------------------|-----------------------------------------------------------------------------|
/// | Standard colors       | `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`       |
/// | Bright colors         | `bright_black`                                                              |
/// |                       | `bright_red`                                                                |
/// |                       | `bright_green`                                                              |
/// |                       | `bright_yellow`                                                             |
/// |                       | `bright_blue`                                                               |
/// |                       | `bright_magenta`                                                            |
/// |                       | `bright_cyan`                                                               |
/// |                       | `bright_white`                                                              |
/// | Bright color shorthands | `b_black`                                                                   |
/// |                       | `b_red`                                                                     |
/// |                       | `b_green`                                                                   |
/// |                       | `b_yellow`                                                                  |
/// |                       | `b_blue`                                                                    |
/// |                       | `b_magenta`                                                                 |
/// |                       | `b_cyan`                                                                    |
/// |                       | `b_white`                                                                   |
/// | Gray colors           | `gray`/`grey`                                                               |
/// |                       | `bright_gray`/`bright_grey`                                                 |
/// |                       | `b_gray`/`b_grey`                                                           |
///
/// Color tags can be nested, `[[/]]` will close the most recently opened color tag.
///
/// # Arguments
/// * `text` - The text to format, can be any type that implements `AsRef<str>`
///
/// # Returns
/// Returns a `String` containing ANSI escape codes that can display colored/formatted text in ANSI-supported terminals.
///
/// # Examples
/// ```
/// use testing::fmt::colorful;
///
/// let formatted = colorful("Hello **world**!");
/// println!("{}", formatted);
///
/// let colored = colorful("[[red]]Red text[[/]] and normal text");
/// println!("{}", colored);
///
/// let nested = colorful("[[blue]]Blue [[green]]Green[[/]] Blue[[/]] normal");
/// println!("{}", nested);
/// ```
pub fn colorful(text: impl AsRef<str>) -> String {
    let text = text.as_ref().trim();
    let mut result = String::new();
    let mut color_stack: VecDeque<String> = VecDeque::new();

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for escape character \
        if chars[i] == '\\' && i + 1 < chars.len() {
            let escaped_char = chars[i + 1];
            // Only escape specific characters
            if matches!(escaped_char, '*' | '<' | '>' | '`' | '_') {
                let mut escaped_text = escaped_char.to_string();
                apply_color_stack(&mut escaped_text, &color_stack);
                result.push_str(&escaped_text);
                i += 2;
                continue;
            }
        }

        // Check for color tag start [[color]]
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            if let Some(end) = find_tag_end(&chars, i) {
                let tag_content: String = chars[i + 2..end].iter().collect();

                // Check if it's a closing tag [[/]]
                if tag_content == "/" {
                    color_stack.pop_back();
                } else {
                    // It's a color tag
                    color_stack.push_back(tag_content.clone());
                }
                i = end + 2;
                continue;
            }
        }

        // Check for bold **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_matching(&chars, i + 2, "**") {
                let bold_text: String = chars[i + 2..end].iter().collect();
                let mut formatted_text = bold_text.bold().to_string();
                apply_color_stack(&mut formatted_text, &color_stack);
                result.push_str(&formatted_text);
                i = end + 2;
                continue;
            }
        }

        // Check for italic *text*
        if chars[i] == '*' {
            if let Some(end) = find_matching(&chars, i + 1, "*") {
                let italic_text: String = chars[i + 1..end].iter().collect();
                let mut formatted_text = italic_text.italic().to_string();
                apply_color_stack(&mut formatted_text, &color_stack);
                result.push_str(&formatted_text);
                i = end + 1;
                continue;
            }
        }

        // Check for underline _text_
        if chars[i] == '_' {
            if let Some(end) = find_matching(&chars, i + 1, "_") {
                let underline_text: String = chars[i + 1..end].iter().collect();
                let mut formatted_text = format!("\x1b[4m{}\x1b[0m", underline_text);
                apply_color_stack(&mut formatted_text, &color_stack);
                result.push_str(&formatted_text);
                i = end + 1;
                continue;
            }
        }

        // Check for angle-bracketed content <text>
        if chars[i] == '<' {
            if let Some(end) = find_matching(&chars, i + 1, ">") {
                // Include the angle brackets in the output
                let angle_text: String = chars[i..=end].iter().collect();
                let mut formatted_text = angle_text.cyan().to_string();
                apply_color_stack(&mut formatted_text, &color_stack);
                result.push_str(&formatted_text);
                i = end + 1;
                continue;
            }
        }

        // Check for inline code `text`
        if chars[i] == '`' {
            if let Some(end) = find_matching(&chars, i + 1, "`") {
                // Include the backticks in the output
                let code_text: String = chars[i..=end].iter().collect();
                let mut formatted_text = code_text.green().to_string();
                apply_color_stack(&mut formatted_text, &color_stack);
                result.push_str(&formatted_text);
                i = end + 1;
                continue;
            }
        }

        // Regular character
        let mut current_char = chars[i].to_string();
        apply_color_stack(&mut current_char, &color_stack);
        result.push_str(&current_char);
        i += 1;
    }

    result
}

// Helper function to find matching delimiter
fn find_matching(chars: &[char], start: usize, delimiter: &str) -> Option<usize> {
    let delim_chars: Vec<char> = delimiter.chars().collect();
    let delim_len = delim_chars.len();

    let mut j = start;
    while j < chars.len() {
        if delim_len == 1 {
            if chars[j] == delim_chars[0] {
                return Some(j);
            }
        } else if j + 1 < chars.len()
            && chars[j] == delim_chars[0]
            && chars[j + 1] == delim_chars[1]
        {
            return Some(j);
        }
        j += 1;
    }
    None
}

// Helper function to find color tag end
fn find_tag_end(chars: &[char], start: usize) -> Option<usize> {
    let mut j = start + 2;
    while j + 1 < chars.len() {
        if chars[j] == ']' && chars[j + 1] == ']' {
            return Some(j);
        }
        j += 1;
    }
    None
}

// Helper function to apply color stack to text
fn apply_color_stack(text: &mut String, color_stack: &VecDeque<String>) {
    let mut result = text.clone();
    for color in color_stack.iter().rev() {
        result = apply_color(&result, color);
    }
    *text = result;
}

// Helper function to apply color to text
fn apply_color(text: impl AsRef<str>, color_name: impl AsRef<str>) -> String {
    let text = text.as_ref();
    let color_name = color_name.as_ref();
    match color_name {
        // Normal colors
        "black" => text.dark_grey().to_string(),
        "red" => text.dark_red().to_string(),
        "green" => text.dark_green().to_string(),
        "yellow" => text.dark_yellow().to_string(),
        "blue" => text.dark_blue().to_string(),
        "magenta" => text.dark_magenta().to_string(),
        "cyan" => text.dark_cyan().to_string(),
        "white" => text.white().to_string(),
        "bright_black" => text.black().to_string(),
        "bright_red" => text.red().to_string(),
        "bright_green" => text.green().to_string(),
        "bright_yellow" => text.yellow().to_string(),
        "bright_blue" => text.blue().to_string(),
        "bright_magenta" => text.magenta().to_string(),
        "bright_cyan" => text.cyan().to_string(),
        "bright_white" => text.white().to_string(),

        // Short aliases for bright colors
        "b_black" => text.black().to_string(),
        "b_red" => text.red().to_string(),
        "b_green" => text.green().to_string(),
        "b_yellow" => text.yellow().to_string(),
        "b_blue" => text.blue().to_string(),
        "b_magenta" => text.magenta().to_string(),
        "b_cyan" => text.cyan().to_string(),
        "b_white" => text.white().to_string(),

        // Gray colors using truecolor
        "gray" | "grey" => text.grey().to_string(),
        "bright_gray" | "bright_grey" => text.white().to_string(),
        "b_gray" | "b_grey" => text.white().to_string(),

        // Default to white if color not recognized
        _ => text.to_string(),
    }
}
