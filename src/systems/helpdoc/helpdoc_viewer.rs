use crate::systems::helpdoc::{get_helpdoc, get_helpdoc_list};
use cli_utils::{
    display::markdown::Markdown,
    env::{helpdoc::get_helpdoc_enabled, locales::current_locales},
};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use rust_i18n::t;
use std::{
    collections::{BTreeMap, HashMap},
    io::{Write, stdout},
};

const TREE_WIDTH: u16 = 25;

struct HelpdocViewer {
    /// Current language
    lang: String,

    /// Document tree structure
    doc_tree: DocTree,

    /// Currently selected document path
    current_doc: String,

    /// Scroll position history
    scroll_history: HashMap<String, usize>,

    /// Current focus area
    focus: FocusArea,

    /// Currently selected node index in tree view
    tree_selection_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusArea {
    Tree,
    Content,
}

#[derive(Debug, Clone)]
struct DocTreeNode {
    /// Node name
    name: String,

    /// Full document path
    path: String,

    /// Child nodes
    children: BTreeMap<String, DocTreeNode>,

    /// Whether it is a document file
    is_document: bool,
}

#[derive(Debug, Clone)]
struct DocTree {
    /// Root node
    root: DocTreeNode,

    /// Flattened document list
    flat_docs: Vec<String>,
}

impl HelpdocViewer {
    fn new(default_doc: &str, lang: &str) -> Self {
        // Build the document tree
        let doc_tree = Self::build_doc_tree();

        // Validate if the default document exists
        let current_doc = if doc_tree.contains_doc(default_doc) {
            default_doc.to_string()
        } else {
            // If the default document does not exist, use the first document
            doc_tree.flat_docs.first().cloned().unwrap_or_default()
        };

        // Calculate the initial tree selection index
        let tree_selection_index = doc_tree
            .flat_docs
            .iter()
            .position(|doc| *doc == current_doc)
            .unwrap_or(0);

        Self {
            lang: lang.to_string(),
            doc_tree,
            current_doc,
            scroll_history: HashMap::new(),
            focus: FocusArea::Content,
            tree_selection_index,
        }
    }

    /// Build document tree
    fn build_doc_tree() -> DocTree {
        // Get all document list
        let doc_list = get_helpdoc_list();

        // Create root node
        let mut root = DocTreeNode {
            name: "helpdoc".to_string(),
            path: "".to_string(),
            children: BTreeMap::new(),
            is_document: false,
        };

        // Build tree structure for each document path
        for doc_path in doc_list {
            Self::add_doc_to_tree(&mut root, doc_path);
        }

        // Build flattened document list
        let flat_docs = Self::flatten_doc_tree(&root);

        DocTree { root, flat_docs }
    }

    /// Add document to tree
    fn add_doc_to_tree(root: &mut DocTreeNode, doc_path: &str) {
        let parts: Vec<&str> = doc_path.split('/').collect();

        // Use recursive helper function to avoid borrowing issues
        Self::add_doc_to_tree_recursive(root, &parts, 0);
    }

    /// Recursively add document to tree
    fn add_doc_to_tree_recursive(node: &mut DocTreeNode, parts: &[&str], depth: usize) {
        if depth >= parts.len() {
            return;
        }

        let part = parts[depth];
        let is_document = depth == parts.len() - 1;
        let current_path = parts[0..=depth].join("/");

        // Check if node already exists, create if not
        if !node.children.contains_key(part) {
            let new_node = DocTreeNode {
                name: part.to_string(),
                path: current_path.clone(),
                children: BTreeMap::new(),
                is_document,
            };
            node.children.insert(part.to_string(), new_node);
        }

        // Get mutable reference to child node
        if let Some(child) = node.children.get_mut(part) {
            // If this is a document node, ensure it's marked as document
            if is_document {
                child.is_document = true;
            }
            // Recursively process next part
            Self::add_doc_to_tree_recursive(child, parts, depth + 1);
        }
    }

    /// Flatten document tree
    fn flatten_doc_tree(node: &DocTreeNode) -> Vec<String> {
        let mut result = Vec::new();

        if node.is_document && !node.path.is_empty() {
            result.push(node.path.clone());
        }

        // Traverse child nodes in alphabetical order
        for child in node.children.values() {
            result.extend(Self::flatten_doc_tree(child));
        }

        result
    }

    /// Get current document content
    fn current_doc_content(&self) -> String {
        let content = get_helpdoc(&self.current_doc, &self.lang);
        if content.is_empty() {
            format!("Document '{}.{}' not found", self.current_doc, self.lang)
        } else {
            content.to_string()
        }
    }

    /// Get formatted document content
    fn formatted_doc_content(&self) -> Vec<String> {
        let content = self.current_doc_content();
        let formatted = content.markdown();
        formatted.lines().map(|s| s.to_string()).collect()
    }

    /// Truncate a string containing ANSI escape sequences, preserving ANSI sequences
    fn truncate_with_ansi(text: &str, max_width: usize) -> String {
        let mut result = String::new();
        let mut current_width = 0;
        let mut chars = text.chars().peekable();
        let mut in_ansi_escape = false;
        let mut ansi_buffer = String::new();

        while let Some(c) = chars.next() {
            if c == '\x1b' {
                ansi_buffer.push(c);
                while let Some(&next_c) = chars.peek() {
                    ansi_buffer.push(next_c);
                    chars.next();

                    if next_c.is_ascii_alphabetic() {
                        break;
                    }
                }

                result.push_str(&ansi_buffer);
                ansi_buffer.clear();
                in_ansi_escape = false;
                continue;
            }

            if in_ansi_escape {
                ansi_buffer.push(c);
                continue;
            }

            let char_width = if c.is_ascii() { 1 } else { 2 };
            if current_width + char_width > max_width {
                break;
            }

            result.push(c);
            current_width += char_width;
        }

        result
    }

    /// Run viewer
    async fn run(&mut self) -> std::io::Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, Hide)?;

        let mut should_exit = false;

        while !should_exit {
            self.draw()?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    should_exit = self.handle_key(key);
                }
            }
        }

        execute!(stdout(), Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }

    /// Draw interface
    fn draw(&self) -> std::io::Result<()> {
        let tree_width = TREE_WIDTH;
        let (width, height) = crossterm::terminal::size()?;

        if height <= 3 || width <= tree_width {
            return Ok(());
        }

        execute!(stdout(), Clear(ClearType::All))?;
        let content_width = width - tree_width - 1;

        // Draw title
        execute!(
            stdout(),
            MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print(format!("{} - {}", t!("helpdoc_viewer.title"), self.lang)),
            ResetColor
        )?;

        // Draw separator line
        for y in 1..height {
            execute!(stdout(), MoveTo(tree_width, y), Print("│"))?;
        }

        // Draw tree view
        self.draw_tree(0, 2, tree_width, height - 4)?;

        // Draw content area
        self.draw_content(tree_width + 1, 2, content_width, height - 4)?;

        // Draw status bar
        self.draw_status_bar(height - 1, width)?;

        stdout().flush()?;
        Ok(())
    }

    /// Draw tree view
    fn draw_tree(&self, x: u16, y: u16, width: u16, height: u16) -> std::io::Result<()> {
        // Draw tree view title
        execute!(
            stdout(),
            MoveTo(x, y - 1),
            SetForegroundColor(Color::Yellow),
            Print("Documents"),
            ResetColor
        )?;

        // Recursively draw tree structure
        let mut line_counter = 0;
        let max_lines = height as usize;

        // Skip root node, start drawing from children
        for child in self.doc_tree.root.children.values() {
            line_counter = self.draw_tree_node(child, x, y, width, 0, line_counter, max_lines)?;
            if line_counter >= max_lines {
                break;
            }
        }

        Ok(())
    }

    /// Recursively draw tree node
    fn draw_tree_node(
        &self,
        node: &DocTreeNode,
        x: u16,
        start_y: u16,
        width: u16,
        depth: usize,
        mut line_counter: usize,
        max_lines: usize,
    ) -> std::io::Result<usize> {
        if line_counter >= max_lines {
            return Ok(line_counter);
        }

        let line_y = start_y + line_counter as u16;
        line_counter += 1;

        // Build indentation and suffix
        let indent = "  ".repeat(depth);
        let suffix = if node.children.is_empty() { "" } else { "/" };

        // If this is the currently selected document, highlight it (white background, black text)
        let is_selected = node.path == self.current_doc;

        if is_selected && self.focus == FocusArea::Tree {
            // Highlight with white background and black text
            execute!(
                stdout(),
                MoveTo(x, line_y),
                SetForegroundColor(Color::Black),
                SetBackgroundColor(Color::White),
                Print(" ".repeat(width as usize)),
                MoveTo(x, line_y),
                SetForegroundColor(Color::Black),
            )?;
        } else {
            // Normal display
            execute!(
                stdout(),
                MoveTo(x, line_y),
                SetForegroundColor(Color::White),
                SetBackgroundColor(Color::Black),
            )?;
        }

        // Display node name
        let display_text = format!("{}  {}{}", indent, node.name, suffix);
        execute!(stdout(), Print(display_text))?;
        execute!(stdout(), ResetColor, SetBackgroundColor(Color::Black))?;

        // Recursively draw child nodes
        if !node.children.is_empty() {
            for child in node.children.values() {
                line_counter = self.draw_tree_node(
                    child,
                    x,
                    start_y,
                    width,
                    depth + 1,
                    line_counter,
                    max_lines,
                )?;
                if line_counter >= max_lines {
                    break;
                }
            }
        }

        Ok(line_counter)
    }

    /// Draw content area
    fn draw_content(&self, x: u16, y: u16, width: u16, height: u16) -> std::io::Result<()> {
        // Draw content area title
        let (fg_color, bg_color) = if self.focus == FocusArea::Content {
            (Color::Black, Color::White)
        } else {
            (Color::Yellow, Color::Black)
        };

        execute!(
            stdout(),
            MoveTo(x, y - 1),
            SetForegroundColor(fg_color),
            SetBackgroundColor(bg_color),
            Print(format!(" Reading `{}` ", self.current_doc)),
            ResetColor
        )?;

        // Get formatted content
        let content_lines = self.formatted_doc_content();
        let scroll_pos = self
            .scroll_history
            .get(&self.current_doc)
            .copied()
            .unwrap_or(0);
        let start_line = scroll_pos.min(content_lines.len().saturating_sub(1));
        let end_line = (start_line + height as usize).min(content_lines.len());

        for (i, line) in content_lines
            .iter()
            .enumerate()
            .take(end_line)
            .skip(start_line)
        {
            let line_y = y + i as u16 - start_line as u16;
            let display_line = Self::truncate_with_ansi(line, width as usize);

            execute!(stdout(), MoveTo(x, line_y), Print(&display_line))?;
        }

        // Display scroll position indicator
        if content_lines.len() > height as usize && content_lines.len() > 0 {
            let scroll_percent = if content_lines.len() > 0 {
                (scroll_pos * 100) / content_lines.len()
            } else {
                0
            };
            execute!(
                stdout(),
                MoveTo(x + width - 5, y - 1),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("{:3}%", scroll_percent)),
                ResetColor
            )?;
        }

        Ok(())
    }

    /// Draw status bar
    fn draw_status_bar(&self, y: u16, width: u16) -> std::io::Result<()> {
        // Draw status bar background
        execute!(
            stdout(),
            MoveTo(0, y),
            SetForegroundColor(Color::Black),
            Print(" ".repeat(width as usize)),
            MoveTo(0, y),
            SetForegroundColor(Color::White),
        )?;

        let status_text = match self.focus {
            FocusArea::Tree => t!("helpdoc_viewer.tree_area_hint").to_string().markdown(),
            FocusArea::Content => t!("helpdoc_viewer.content_area_hint")
                .to_string()
                .markdown(),
        }
        .to_string();

        let truncated_text = Self::truncate_with_ansi(&status_text, width as usize);
        execute!(stdout(), Print(&truncated_text))?;
        execute!(stdout(), ResetColor)?;

        Ok(())
    }

    /// Handle key input
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char(' ') => self.toggle_focus(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Char('g') if key.modifiers == KeyModifiers::NONE => self.go_to_top(),
            KeyCode::Char('G') if key.modifiers == KeyModifiers::SHIFT => self.go_to_bottom(),
            KeyCode::Enter => self.select_item(),
            _ => {}
        }
        false
    }

    /// Toggle focus area
    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusArea::Tree => FocusArea::Content,
            FocusArea::Content => FocusArea::Tree,
        };
    }

    /// Move left
    fn move_left(&mut self) {
        if self.focus == FocusArea::Content {
            self.focus = FocusArea::Tree;
        }
    }

    /// Move right
    fn move_right(&mut self) {
        if self.focus == FocusArea::Tree {
            self.focus = FocusArea::Content;
        }
    }

    /// Move up
    fn move_up(&mut self) {
        match self.focus {
            FocusArea::Tree => self.previous_doc(),
            FocusArea::Content => self.scroll_up(),
        }
    }

    /// Move down
    fn move_down(&mut self) {
        match self.focus {
            FocusArea::Tree => self.next_doc(),
            FocusArea::Content => self.scroll_down(),
        }
    }

    /// Scroll to top
    fn go_to_top(&mut self) {
        match self.focus {
            FocusArea::Content => {
                self.scroll_history.insert(self.current_doc.clone(), 0);
            }
            FocusArea::Tree => {
                // Select first document
                self.tree_selection_index = 0;
                if let Some(first_doc) = self.doc_tree.flat_docs.first() {
                    self.current_doc = first_doc.clone();
                }
            }
        }
    }

    /// Scroll to bottom
    fn go_to_bottom(&mut self) {
        match self.focus {
            FocusArea::Content => {
                let content_lines = self.formatted_doc_content();
                if content_lines.len() > 10 {
                    self.scroll_history
                        .insert(self.current_doc.clone(), content_lines.len() - 10);
                }
            }
            FocusArea::Tree => {
                // Select last document
                self.tree_selection_index = self.doc_tree.flat_docs.len().saturating_sub(1);
                if let Some(last_doc) = self.doc_tree.flat_docs.last() {
                    self.current_doc = last_doc.clone();
                }
            }
        }
    }

    /// Select current item
    fn select_item(&mut self) {
        match self.focus {
            FocusArea::Tree => {
                // Update current document to the one selected in tree view
                if let Some(doc) = self.doc_tree.flat_docs.get(self.tree_selection_index) {
                    self.current_doc = doc.clone();
                }
                // Switch focus to content area
                self.focus = FocusArea::Content;
            }
            _ => {}
        }
    }

    /// Previous document
    fn previous_doc(&mut self) {
        if self.tree_selection_index > 0 {
            self.tree_selection_index -= 1;
            if let Some(doc) = self.doc_tree.flat_docs.get(self.tree_selection_index) {
                self.current_doc = doc.clone();
                // Reset scroll position
                self.scroll_history.remove(&self.current_doc);
            }
        }
    }

    /// Next document
    fn next_doc(&mut self) {
        if self.tree_selection_index + 1 < self.doc_tree.flat_docs.len() {
            self.tree_selection_index += 1;
            if let Some(doc) = self.doc_tree.flat_docs.get(self.tree_selection_index) {
                self.current_doc = doc.clone();
                // Reset scroll position
                self.scroll_history.remove(&self.current_doc);
            }
        }
    }

    /// Scroll up
    fn scroll_up(&mut self) {
        let current_scroll = self
            .scroll_history
            .get(&self.current_doc)
            .copied()
            .unwrap_or(0);
        if current_scroll > 0 {
            self.scroll_history
                .insert(self.current_doc.clone(), current_scroll - 1);
        }
    }

    /// Scroll down
    fn scroll_down(&mut self) {
        let content_lines = self.formatted_doc_content();
        let current_scroll = self
            .scroll_history
            .get(&self.current_doc)
            .copied()
            .unwrap_or(0);
        if current_scroll + 1 < content_lines.len() {
            self.scroll_history
                .insert(self.current_doc.clone(), current_scroll + 1);
        }
    }
}

impl DocTree {
    /// Check if document exists
    fn contains_doc(&self, doc_path: &str) -> bool {
        self.flat_docs.contains(&doc_path.to_string())
    }
}

/// Display help document viewer
pub async fn display_with_lang(default_focus_doc: &str, lang: &str) {
    if get_helpdoc_enabled() {
        let mut viewer = HelpdocViewer::new(default_focus_doc, lang);

        if let Err(e) = viewer.run().await {
            eprintln!("Error running helpdoc viewer: {}", e);
        }
    } else {
        let content = get_helpdoc(default_focus_doc, lang).markdown();
        println!("{}", content)
    }
}

/// Display help document viewer
pub async fn display(default_focus_doc: &str) {
    display_with_lang(default_focus_doc, current_locales().as_str()).await;
}
