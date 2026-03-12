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
