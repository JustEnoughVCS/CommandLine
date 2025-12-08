use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

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
}

impl Editor {
    fn new(file_path: PathBuf) -> io::Result<Self> {
        let content = if file_path.exists() {
            fs::read_to_string(&file_path)?
                .lines()
                .map(|s| s.to_string())
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
            "{} - {} lines{}",
            self.file_path.display(),
            self.content.len(),
            if self.modified { " *" } else { "" }
        );

        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(style::SetBackgroundColor(Color::DarkBlue))?;
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
            eprintln!("Failed to enable raw mode: {}", e);
            return Err(e);
        }

        if let Err(e) = execute!(stdout, EnterAlternateScreen) {
            disable_raw_mode().ok();
            eprintln!("Failed to enter alternate screen: {}", e);
            return Err(e);
        }

        // Initial render
        if let Err(e) = self.render(&mut stdout) {
            self.cleanup_terminal(&mut stdout)?;
            return Err(e);
        }

        // Event loop
        let result = loop {
            match event::read() {
                Ok(Event::Key(key_event)) => {
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

    fn handle_key_event(&mut self, key_event: KeyEvent, stdout: &mut io::Stdout) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Err(e) = self.save() {
                    eprintln!("Failed to save file: {}", e);
                    // Continue editing even if save fails
                } else {
                    self.show_message("File saved successfully", stdout)?;
                }
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
                    _ => self.insert_char(c),
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
                    self.show_message(
                        "Unsaved changes! Press Ctrl+S to save or Esc again to exit",
                        stdout,
                    )?;
                    // Don't exit immediately, wait for second Esc
                } else {
                    self.should_exit = true;
                }
            }
            _ => {}
        }

        if let Err(e) = self.render(stdout) {
            eprintln!("Render error: {}", e);
            return Err(e);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let args = JustEnoughVcsInputer::parse();

    // Check if a file argument was provided
    let file_path = match args.file {
        Some(path) => path,
        None => {
            eprintln!("Error: No file path provided");
            std::process::exit(1);
        }
    };

    // Perform precheck
    match precheck(file_path) {
        Ok(full_path) => {
            if let Err(e) = open_editor(full_path).await {
                eprintln!("Editor error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("File error: {}", e);
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

    // Create file if it doesn't exist
    if !full_path.exists() {
        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Create empty file
        fs::write(&full_path, "")?;
    }

    // Check if it's a file (or we just created it as a file)
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
                eprintln!("Editor error: {}", e);
                return Err(e);
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to initialize editor: {}", e);
            Err(e)
        }
    }
}
