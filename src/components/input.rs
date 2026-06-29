use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

use crate::config::ParsedEditorKeybindings;

const PROMPT_PREFIX: &str = "› ";

pub struct Input {
    pub value: String,
    cursor: usize,
    scroll: usize,
    wrap_width: usize,
    preferred_col: Option<usize>,
    keys: ParsedEditorKeybindings,
}

impl Input {
    pub fn new(keys: ParsedEditorKeybindings) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            scroll: 0,
            wrap_width: 80,
            preferred_col: None,
            keys,
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> bool {
        if self.keys.newline.matches(key) || self.keys.newline_alt.matches(key) {
            self.insert_char('\n');
            self.preferred_col = None;
            return true;
        }
        if self.keys.clear.matches(key) {
            self.value.clear();
            self.cursor = 0;
            self.scroll = 0;
            self.preferred_col = None;
            return true;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }

        match key.code {
            KeyCode::Up => {
                let (rows, starts, crow, ccol) = self.visual_lines(self.wrap_width.max(1));
                if crow > 0 {
                    let col = self.preferred_col.unwrap_or(ccol);
                    self.preferred_col = Some(col);
                    let prev_len = rows[crow - 1].chars().count();
                    self.cursor = starts[crow - 1] + col.min(prev_len);
                }
                return true;
            }
            KeyCode::Down => {
                let (rows, starts, crow, ccol) = self.visual_lines(self.wrap_width.max(1));
                if crow + 1 < rows.len() {
                    let col = self.preferred_col.unwrap_or(ccol);
                    self.preferred_col = Some(col);
                    let next_len = rows[crow + 1].chars().count();
                    self.cursor = starts[crow + 1] + col.min(next_len);
                }
                return true;
            }
            _ => {}
        }

        self.preferred_col = None;
        match key.code {
            KeyCode::Char(ch) if !ch.is_control() => {
                self.insert_char(ch);
                true
            }
            KeyCode::Enter => {
                self.value.clear();
                self.cursor = 0;
                true
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    let bi = self.byte_index(self.cursor);
                    self.value.remove(bi);
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor < self.char_len() {
                    let bi = self.byte_index(self.cursor);
                    self.value.remove(bi);
                }
                true
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                true
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.char_len());
                true
            }
            KeyCode::Home => {
                let lengths = self.logical_line_lengths();
                let (line, _) = self.logical_line_of(self.cursor);
                self.cursor = self.line_start_char(&lengths, line);
                true
            }
            KeyCode::End => {
                let lengths = self.logical_line_lengths();
                let (line, _) = self.logical_line_of(self.cursor);
                self.cursor = self.line_start_char(&lengths, line) + lengths[line];
                true
            }
            _ => false,
        }
    }

    pub fn needed_height(&self, area_width: u16) -> u16 {
        let inner_width = area_width.saturating_sub(2).max(1) as usize;
        let rows = self.visual_lines(inner_width).0.len() as u16;
        rows.saturating_add(2)
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().fg(Color::Cyan);
        let inner = block.inner(area);
        frame.render_widget(&block, area);

        let inner_width = inner.width as usize;
        self.wrap_width = inner_width;
        let (visual, _starts, cursor_row, cursor_col) = self.visual_lines(inner_width);

        let visible = inner.height.max(1) as usize;
        let max_scroll = visual.len().saturating_sub(visible);
        if cursor_row < self.scroll {
            self.scroll = cursor_row;
        } else if cursor_row >= self.scroll + visible {
            self.scroll = cursor_row + 1 - visible;
        }
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        let indent = " ".repeat(PROMPT_PREFIX.chars().count());
        let lines: Vec<Line> = if self.value.is_empty() {
            vec![Line::from(vec![
                Span::styled(PROMPT_PREFIX, Style::default().fg(Color::Cyan)),
                Span::styled("Ask nixy anything…", Style::default().fg(Color::DarkGray)),
            ])]
        } else {
            visual
                .iter()
                .enumerate()
                .skip(self.scroll)
                .take(visible)
                .map(|(i, s)| {
                    if i == 0 {
                        Line::from(vec![
                            Span::styled(PROMPT_PREFIX, Style::default().fg(Color::Cyan)),
                            Span::raw(s.as_str()).fg(Color::White),
                        ])
                    } else {
                        Line::from(vec![
                            Span::raw(indent.as_str()),
                            Span::raw(s.as_str()).fg(Color::White),
                        ])
                    }
                })
                .collect()
        };
        frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner);

        let prefix_len = PROMPT_PREFIX.chars().count();
        let cx =
            inner.x + (prefix_len + cursor_col).min(inner.width.saturating_sub(1) as usize) as u16;
        let cy = inner.y + (cursor_row - self.scroll) as u16;
        frame.set_cursor_position((cx, cy));
    }

    fn char_len(&self) -> usize {
        self.value.chars().count()
    }

    fn byte_index(&self, char_index: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_index)
            .map(|(b, _)| b)
            .unwrap_or_else(|| self.value.len())
    }

    fn insert_char(&mut self, ch: char) {
        let bi = self.byte_index(self.cursor);
        self.value.insert(bi, ch);
        self.cursor += 1;
    }

    fn logical_line_of(&self, cursor: usize) -> (usize, usize) {
        let mut line = 0usize;
        let mut line_start = 0usize;
        for (i, ch) in self.value.chars().enumerate() {
            if i == cursor {
                return (line, i - line_start);
            }
            if ch == '\n' {
                line += 1;
                line_start = i + 1;
            }
        }
        (line, cursor - line_start)
    }

    fn logical_line_lengths(&self) -> Vec<usize> {
        self.value.split('\n').map(|s| s.chars().count()).collect()
    }

    fn line_start_char(&self, lengths: &[usize], line_idx: usize) -> usize {
        lengths.iter().take(line_idx).map(|&l| l + 1).sum()
    }

    fn visual_lines(&self, inner_width: usize) -> (Vec<String>, Vec<usize>, usize, usize) {
        let width = inner_width.max(1);
        let content_width = width.saturating_sub(PROMPT_PREFIX.chars().count()).max(1);

        let mut rows: Vec<String> = vec![String::new()];
        let mut starts: Vec<usize> = vec![0];
        let mut capacity = content_width;
        let mut cursor_row = 0;
        let mut cursor_col = 0;
        let mut found_cursor = false;

        for (i, ch) in self.value.chars().enumerate() {
            if i == self.cursor {
                cursor_row = rows.len() - 1;
                cursor_col = rows.last().unwrap().chars().count();
                found_cursor = true;
            }
            if ch == '\n' {
                rows.push(String::new());
                starts.push(i + 1);
                capacity = content_width;
            } else {
                if capacity == 0 {
                    rows.push(String::new());
                    starts.push(i);
                    capacity = content_width;
                }
                rows.last_mut().unwrap().push(ch);
                capacity -= 1;
            }
        }

        if !found_cursor {
            cursor_row = rows.len() - 1;
            cursor_col = rows.last().unwrap().chars().count();
        }

        (rows, starts, cursor_row, cursor_col)
    }
}
