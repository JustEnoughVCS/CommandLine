use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, command};
use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{self, Color, Print, SetForegroundColor},
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use just_enough_vcs_cli::utils::display::display_width;
use just_enough_vcs_cli::utils::display::md;
use just_enough_vcs_cli::utils::env::current_locales;
use rust_i18n::set_locale;
use rust_i18n::t;
#[cfg(windows)]
use tokio::time::Instant;

// Import i18n files
rust_i18n::i18n!("locales", fallback = "en");

#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]
struct JustEnoughVcsInputer {
    /// File to edit
    file: Option<PathBuf>,
}

struct Editor {
    content: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    selection_start: Option<(usize, usize)>,
    is_selecting: bool,
    file_path: PathBuf,
    modified: bool,
    terminal_size: (u16, u16),
    should_exit: bool,
    #[cfg(windows)]
    last_key_event: Option<(KeyCode, KeyModifiers, Instant)>,
    #[cfg(windows)]
    ime_composing: bool,
}

impl Editor {
    fn new(file_path: PathBuf) -> io::Result<Self> {
        let content = if file_path.exists() {
            fs::read_to_string(&file_path)?
                .lines()
                .map(|line| line.to_string())
                .collect()
        } else {
            vec![String::new()]
        };

        let (width, height) = terminal::size().unwrap_or((80, 24));

        Ok(Self {
            content,
            cursor_x: 0,
            cursor_y: 0,
            selection_start: None,
            is_selecting: false,
            file_path,
            modified: false,
            terminal_size: (width, height),
            should_exit: false,
            #[cfg(windows)]
            last_key_event: None,
            #[cfg(windows)]
            ime_composing: false,
        })
    }

    fn show_message(&mut self, message: &str, stdout: &mut io::Stdout) -> io::Result<()> {
        let (width, height) = self.terminal_size;
        let message_line = height - 2;

        // Save current cursor position
        stdout.queue(MoveTo(0, message_line))?;
        stdout.queue(Clear(ClearType::CurrentLine))?;

        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(style::SetBackgroundColor(Color::DarkBlue))?;
        let display_message = if display_width(message) > width as usize {
            // Find the maximum number of characters that fit within width
            let mut _chars_count = 0;
            let mut current_width = 0;
            let mut byte_pos = 0;
            for c in message.chars() {
                let char_width = if c.is_ascii() { 1 } else { 2 };
                if current_width + char_width > width as usize {
                    break;
                }
                current_width += char_width;
                _chars_count += 1;
                byte_pos += c.len_utf8();
            }
            &message[..byte_pos]
        } else {
            &message
        };
        stdout.queue(Print(display_message))?;
        stdout.queue(style::ResetColor)?;

        // Restore cursor position - calculate display position
        let start_line = if self.cursor_y >= (height - 2) as usize {
            self.cursor_y.saturating_sub((height - 2) as usize)
        } else {
            0
        };
        let cursor_screen_y = self.cursor_y.saturating_sub(start_line) as u16;

        // Calculate cursor x position based on display width
        let line = &self.content[self.cursor_y];
        let mut cursor_display_x = 0;
        let mut chars_processed = 0;

        for c in line.chars() {
            if chars_processed >= self.cursor_x {
                break;
            }
            cursor_display_x += if c.is_ascii() { 1 } else { 2 };
            chars_processed += 1;
        }

        let cursor_screen_x = cursor_display_x.min(width as usize) as u16;
        stdout.queue(MoveTo(cursor_screen_x, cursor_screen_y))?;

        stdout.flush()?;
        Ok(())
    }

    fn save(&mut self) -> io::Result<()> {
        let content = self.content.join("\n");
        fs::write(&self.file_path, content)?;
        self.modified = false;
        Ok(())
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        let new_y = (self.cursor_y as i32 + dy).max(0) as usize;
        let new_y = new_y.min(self.content.len().saturating_sub(1));

        if new_y != self.cursor_y {
            self.cursor_y = new_y;
            // Limit cursor_x to number of characters in the line
            self.cursor_x = self
                .cursor_x
                .min(self.content[self.cursor_y].chars().count());
        } else if dx != 0 {
            let new_x = (self.cursor_x as i32 + dx).max(0) as usize;
            // Limit to number of characters in the line
            self.cursor_x = new_x.min(self.content[self.cursor_y].chars().count());
        }
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor_y >= self.content.len() {
            self.content.push(String::new());
        }

        let line = &mut self.content[self.cursor_y];

        // Convert character position to byte position
        let byte_pos = if self.cursor_x == 0 {
            0
        } else {
            line.char_indices()
                .nth(self.cursor_x - 1)
                .map(|(i, _)| i + line.chars().nth(self.cursor_x - 1).unwrap().len_utf8())
                .unwrap_or(line.len())
        };

        if byte_pos <= line.len() {
            line.insert(byte_pos, c);
            self.cursor_x += 1;
            self.modified = true;
        }
    }

    fn delete_char(&mut self) {
        if self.cursor_x > 0 && self.cursor_y < self.content.len() {
            let line = &mut self.content[self.cursor_y];

            // Convert character position to byte position for removal
            let byte_pos = if self.cursor_x == 1 {
                0
            } else {
                line.char_indices()
                    .nth(self.cursor_x - 2)
                    .map(|(i, _)| i + line.chars().nth(self.cursor_x - 2).unwrap().len_utf8())
                    .unwrap_or(line.len())
            };

            // Find the character to remove
            if let Some(c) = line.chars().nth(self.cursor_x - 1) {
                let char_len = c.len_utf8();
                line.drain(byte_pos..byte_pos + char_len);
                self.cursor_x -= 1;
                self.modified = true;
            }
        } else if self.cursor_x == 0 && self.cursor_y > 0 {
            // Merge with previous line
            let current_line = self.content.remove(self.cursor_y);
            self.cursor_y -= 1;
            // Set cursor to end of previous line (in characters)
            self.cursor_x = self.content[self.cursor_y].chars().count();
            self.content[self.cursor_y].push_str(&current_line);
            self.modified = true;
        }
    }

    fn new_line(&mut self) {
        if self.cursor_y >= self.content.len() {
            self.content.push(String::new());
            self.cursor_y = self.content.len() - 1;
            self.cursor_x = 0;
        } else {
            let line = self.content[self.cursor_y].clone();

            // Convert character position to byte position for splitting
            let byte_pos = if self.cursor_x == 0 {
                0
            } else {
                line.char_indices()
                    .nth(self.cursor_x - 1)
                    .map(|(i, _)| i + line.chars().nth(self.cursor_x - 1).unwrap().len_utf8())
                    .unwrap_or(line.len())
            };

            let (left, right) = line.split_at(byte_pos);

            self.content[self.cursor_y] = left.to_string();
            self.content.insert(self.cursor_y + 1, right.to_string());

            self.cursor_y += 1;
            self.cursor_x = 0;
        }
        self.modified = true;
    }

    fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Clear screen
        stdout.queue(Clear(ClearType::All))?;

        // Calculate visible range
        let (width, height) = self.terminal_size;
        let start_line = if self.cursor_y >= (height - 2) as usize {
            self.cursor_y.saturating_sub((height - 2) as usize)
        } else {
            0
        };

        let end_line = (start_line + (height - 2) as usize).min(self.content.len());

        // Render content
        for (i, line) in self.content[start_line..end_line].iter().enumerate() {
            stdout.queue(MoveTo(0, i as u16))?;

            // Truncate line if too long based on display width
            let display_line = if display_width(line) > width as usize {
                // Find the maximum number of characters that fit within width
                let mut current_width = 0;
                let mut byte_pos = 0;
                for c in line.chars() {
                    let char_width = if c.is_ascii() { 1 } else { 2 };
                    if current_width + char_width > width as usize {
                        break;
                    }
                    current_width += char_width;
                    byte_pos += c.len_utf8();
                }
                &line[..byte_pos]
            } else {
                line
            };

            stdout.queue(Print(display_line))?;
        }

        // Render status bar
        let status_line = height - 1;
        stdout.queue(MoveTo(0, status_line))?;
        stdout.queue(Clear(ClearType::CurrentLine))?;

        let status = format!(
            "{} - {}{}{} {}",
            self.file_path.display(),
            self.content.len(),
            t!("jvii.status.lines"),
            if self.modified {
                t!("jvii.messages.modified").to_string()
            } else {
                "".to_string()
            },
            md(t!("jvii.hints"))
        );

        stdout.queue(SetForegroundColor(Color::Black))?;
        stdout.queue(style::SetBackgroundColor(Color::White))?;
        let display_status = if display_width(&status) > width as usize {
            // Find the maximum number of characters that fit within width
            let mut current_width = 0;
            let mut byte_pos = 0;
            for c in status.chars() {
                let char_width = if c.is_ascii() { 1 } else { 2 };
                if current_width + char_width > width as usize {
                    break;
                }
                current_width += char_width;
                byte_pos += c.len_utf8();
            }
            &status[..byte_pos]
        } else {
            &status
        };
        stdout.queue(Print(display_status))?;

        // Reset colors
        stdout.queue(style::ResetColor)?;

        // Position cursor - calculate display position based on display width
        let cursor_screen_y = self.cursor_y.saturating_sub(start_line) as u16;

        // Calculate cursor x position based on display width of characters before cursor
        let line = &self.content[self.cursor_y];
        let mut cursor_display_x = 0;
        let mut chars_processed = 0;

        for c in line.chars() {
            if chars_processed >= self.cursor_x {
                break;
            }
            cursor_display_x += if c.is_ascii() { 1 } else { 2 };
            chars_processed += 1;
        }

        let cursor_screen_x = cursor_display_x.min(width as usize) as u16;

        stdout.queue(MoveTo(cursor_screen_x, cursor_screen_y))?;

        stdout.flush()?;
        Ok(())
    }

    fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Setup terminal with error handling
        if let Err(e) = enable_raw_mode() {
            eprintln!(
                "{}",
                t!("jvii.errors.raw_mode_error", error = e.to_string())
            );
            return Err(e);
        }

        if let Err(e) = execute!(stdout, EnterAlternateScreen) {
            disable_raw_mode().ok();
            eprintln!(
                "{}",
                t!("jvii.errors.alternate_screen_error", error = e.to_string())
            );
            return Err(e);
        }

        // Clear input buffer to avoid leftover keystrokes from command execution
        self.clear_input_buffer()?;

        // Initial render
        if let Err(e) = self.render(&mut stdout) {
            self.cleanup_terminal(&mut stdout)?;
            return Err(e);
        }

        // Event loop
        let result = loop {
            match event::read() {
                Ok(Event::Key(key_event)) => {
                    // Windows-specific input handling for IME and duplicate events
                    #[cfg(windows)]
                    {
                        // Skip key release events (we only care about presses)

                        use crossterm::event::KeyEventKind;
                        if matches!(key_event.kind, KeyEventKind::Release) {
                            continue;
                        }

                        // Handle IME composition
                        if self.should_skip_ime_event(&key_event) {
                            continue;
                        }

                        // Skip duplicate events
                        if self.is_duplicate_event(&key_event) {
                            continue;
                        }
                    }

                    if let Err(e) = self.handle_key_event(key_event, &mut stdout) {
                        break Err(e);
                    }

                    if self.should_exit {
                        break Ok(());
                    }
                }
                Ok(Event::Resize(width, height)) => {
                    self.terminal_size = (width, height);
                    if let Err(e) = self.render(&mut stdout) {
                        break Err(e);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    break Err(e);
                }
            }
        };

        // Cleanup terminal
        self.cleanup_terminal(&mut stdout)?;
        result
    }

    fn cleanup_terminal(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(stdout, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn clear_input_buffer(&self) -> io::Result<()> {
        // Try to read and discard any pending events in the buffer
        while event::poll(Duration::from_millis(0))? {
            let _ = event::read()?;
        }
        Ok(())
    }

    #[cfg(windows)]
    fn is_duplicate_event(&mut self, key_event: &KeyEvent) -> bool {
        let now = Instant::now();
        let current_event = (key_event.code.clone(), key_event.modifiers, now);

        // Check if this is the same event that just happened
        if let Some((last_code, last_modifiers, last_time)) = &self.last_key_event {
            if *last_code == key_event.code
                && *last_modifiers == key_event.modifiers
                && now.duration_since(*last_time) < Duration::from_millis(20)
            // Reduced to 20ms for better responsiveness
            {
                // This is likely a duplicate event from IME or Windows input handling
                return true;
            }
        }

        // Update last event
        self.last_key_event = Some(current_event);
        false
    }

    #[cfg(windows)]
    fn should_skip_ime_event(&mut self, key_event: &KeyEvent) -> bool {
        // Check for IME composition markers
        match &key_event.code {
            KeyCode::Char(c) => {
                // IME composition often produces control characters or special sequences
                let c_u32 = *c as u32;

                // Check for IME composition start/end markers
                // Some IMEs use special characters or sequences
                if c_u32 == 0x16 || c_u32 == 0x17 || c_u32 == 0x18 {
                    // These are common IME control characters
                    self.ime_composing = true;
                    return true;
                }

                // Check for dead keys or composition characters
                if c_u32 < 0x20 || (c_u32 >= 0x80 && c_u32 < 0xA0) {
                    // Control characters or C1 control codes
                    return true;
                }

                // If we were composing and get a normal character, check if it's part of composition
                if self.ime_composing {
                    // Reset composition state when we get a printable character
                    if c.is_ascii_graphic() || c.is_alphanumeric() {
                        self.ime_composing = false;
                    } else {
                        return true;
                    }
                }

                false
            }
            _ => false,
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, stdout: &mut io::Stdout) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Err(e) = self.save() {
                    eprintln!("{}", t!("jvii.errors.save_error", error = e.to_string()));
                    // Continue editing even if save fails
                } else {
                    self.show_message(&t!("jvii.messages.file_saved"), stdout)?;
                }
            }
            KeyCode::Char('v') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Handle Ctrl+V paste - skip normal character insertion
                // On Windows, Ctrl+V might also generate a 'v' character event
                // We'll handle paste separately if needed
                self.is_selecting = false;
                self.selection_start = None;
                // For now, just ignore Ctrl+V to prevent extra 'v' character
                return Ok(());
            }
            KeyCode::Char(c) => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.is_selecting = true;
                    if self.selection_start.is_none() {
                        self.selection_start = Some((self.cursor_x, self.cursor_y));
                    }
                } else {
                    self.is_selecting = false;
                    self.selection_start = None;
                }

                // Handle special characters
                match c {
                    '\n' | '\r' => self.new_line(),
                    _ => {
                        self.insert_char(c);
                    }
                }
            }
            KeyCode::Backspace => {
                self.delete_char();
                self.is_selecting = false;
                self.selection_start = None;
            }
            KeyCode::Enter => {
                self.new_line();
                self.is_selecting = false;
                self.selection_start = None;
            }
            KeyCode::Left => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.is_selecting = true;
                    if self.selection_start.is_none() {
                        self.selection_start = Some((self.cursor_x, self.cursor_y));
                    }
                } else {
                    self.is_selecting = false;
                    self.selection_start = None;
                }
                self.move_cursor(-1, 0);
            }
            KeyCode::Right => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.is_selecting = true;
                    if self.selection_start.is_none() {
                        self.selection_start = Some((self.cursor_x, self.cursor_y));
                    }
                } else {
                    self.is_selecting = false;
                    self.selection_start = None;
                }
                self.move_cursor(1, 0);
            }
            KeyCode::Up => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.is_selecting = true;
                    if self.selection_start.is_none() {
                        self.selection_start = Some((self.cursor_x, self.cursor_y));
                    }
                } else {
                    self.is_selecting = false;
                    self.selection_start = None;
                }
                self.move_cursor(0, -1);
            }
            KeyCode::Down => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.is_selecting = true;
                    if self.selection_start.is_none() {
                        self.selection_start = Some((self.cursor_x, self.cursor_y));
                    }
                } else {
                    self.is_selecting = false;
                    self.selection_start = None;
                }
                self.move_cursor(0, 1);
            }
            KeyCode::Esc => {
                if self.modified {
                    self.show_message(&t!("jvii.messages.unsaved_changes"), stdout)?;
                    // Don't exit immediately, wait for second Esc
                } else {
                    self.should_exit = true;
                }
            }
            _ => {}
        }

        if let Err(e) = self.render(stdout) {
            eprintln!("{}", t!("jvii.errors.render_error", error = e.to_string()));
            return Err(e);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // Init i18n
    set_locale(&current_locales());

    // Windows specific initialization for colored output
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let args = JustEnoughVcsInputer::parse();

    // Check if a file argument was provided
    let file_path = match args.file {
        Some(path) => path,
        None => {
            eprintln!("{}", t!("jvii.errors.no_file_path"));
            std::process::exit(1);
        }
    };

    // Perform precheck
    match precheck(file_path) {
        Ok(full_path) => {
            if let Err(e) = open_editor(full_path).await {
                eprintln!("{}", t!("jvii.errors.editor_error", error = e.to_string()));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}", t!("jvii.errors.file_error", error = e.to_string()));
            std::process::exit(1);
        }
    }
}

fn precheck(file_path: PathBuf) -> Result<PathBuf, std::io::Error> {
    // Get the current directory
    let current_dir = env::current_dir()?;

    // Build the full path
    let full_path = if file_path.is_absolute() {
        file_path
    } else {
        current_dir.join(&file_path)
    };

    // Check if the file exists
    if !full_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File does not exist: {}", full_path.display()),
        ));
    }

    // Check if it's a file
    if !full_path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a file: {}", full_path.display()),
        ));
    }

    Ok(full_path)
}

async fn open_editor(file: PathBuf) -> io::Result<()> {
    match Editor::new(file) {
        Ok(mut editor) => {
            let result = editor.run();

            // Always try to cleanup terminal even if there was an error
            if let Err(e) = result {
                eprintln!("{}", t!("jvii.errors.editor_error", error = e.to_string()));
                return Err(e);
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("{}", t!("jvii.errors.init_error", error = e.to_string()));
            Err(e)
        }
    }
}
