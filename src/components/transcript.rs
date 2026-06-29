use ratatui::layout::{Margin, Rect};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

const MAX_MESSAGES: usize = 1000;
const SEPARATOR: &str = "";

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Agent,
    System,
}

impl Role {
    fn prefix(self) -> &'static str {
        match self {
            Role::User => "› ",
            Role::Agent => "» ",
            Role::System => "· ",
        }
    }

    fn color(self) -> Color {
        match self {
            Role::User => Color::Cyan,
            Role::Agent => Color::Magenta,
            Role::System => Color::DarkGray,
        }
    }
}

#[derive(Clone)]
pub struct Message {
    role: Role,
    text: String,
}

impl Message {
    pub fn user(text: String) -> Self {
        Self {
            role: Role::User,
            text,
        }
    }
}

pub struct Transcript {
    messages: Vec<Message>,
    scroll: usize,
    total: usize,
    visible: usize,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll: 0,
            total: 0,
            visible: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn visible_lines(&self) -> usize {
        self.visible
    }

    pub fn push(&mut self, message: Message) {
        if self.messages.len() >= MAX_MESSAGES {
            self.messages.remove(0);
        }
        self.messages.push(message);
        self.scroll = 0;
    }

    pub fn scroll_up(&mut self, by: usize) {
        let max = self.max_scroll();
        self.scroll = (self.scroll + by).min(max);
    }

    pub fn scroll_down(&mut self, by: usize) {
        self.scroll = self.scroll.saturating_sub(by);
    }

    pub fn scroll_top(&mut self) {
        self.scroll = self.max_scroll();
    }

    pub fn scroll_bottom(&mut self) {
        self.scroll = 0;
    }

    fn max_scroll(&self) -> usize {
        self.total.saturating_sub(self.visible)
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let inner = area.inner(Margin::new(1, 0));
        let width = inner.width as usize;
        let visible = inner.height as usize;

        let lines = render_lines(&self.messages, width);
        let total = lines.len();
        self.total = total;
        self.visible = visible;

        let max_scroll = total.saturating_sub(visible);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        let start = max_scroll.saturating_sub(self.scroll);
        let end = (start + visible).min(total);
        let view = lines[start.min(total)..end].to_vec();

        frame.render_widget(Paragraph::new(view), inner);
    }
}

fn render_lines(messages: &[Message], width: usize) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    for (i, message) in messages.iter().enumerate() {
        if i > 0 {
            out.push(Line::from(SEPARATOR));
        }
        let wrapped = wrap_text(&message.text, width);
        let prefix = message.role.prefix();
        let color = message.role.color();
        let indent = " ".repeat(prefix.chars().count());
        for (j, row) in wrapped.into_iter().enumerate() {
            if j == 0 {
                out.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color)),
                    Span::raw(row).fg(Color::White),
                ]));
            } else {
                out.push(Line::from(vec![
                    Span::raw(indent.clone()),
                    Span::raw(row).fg(Color::White),
                ]));
            }
        }
    }
    out
}

impl Default for Transcript {
    fn default() -> Self {
        Self::new()
    }
}

fn wrap_text(s: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut rows = Vec::new();
    for line in s.split('\n') {
        if line.is_empty() {
            rows.push(String::new());
            continue;
        }
        let mut cur = String::new();
        let mut cap = width;
        for ch in line.chars() {
            if cap == 0 {
                rows.push(std::mem::take(&mut cur));
                cap = width;
            }
            cur.push(ch);
            cap -= 1;
        }
        rows.push(cur);
    }
    if rows.is_empty() {
        rows.push(String::new());
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_splits_long_lines() {
        let rows = wrap_text("abcdef", 3);
        assert_eq!(rows, vec!["abc", "def"]);
    }

    #[test]
    fn wrap_preserves_newlines() {
        let rows = wrap_text("ab\ncd", 10);
        assert_eq!(rows, vec!["ab", "cd"]);
    }

    #[test]
    fn wrap_empty_input_yields_one_row() {
        let rows = wrap_text("", 10);
        assert_eq!(rows, vec![""]);
    }

    #[test]
    fn push_pins_to_bottom() {
        let mut t = Transcript::new();
        t.scroll_up(5);
        t.push(Message::user("hi".to_string()));
        assert_eq!(t.scroll, 0);
        assert!(!t.is_empty());
    }

    #[test]
    fn scroll_down_does_not_underflow() {
        let mut t = Transcript::new();
        t.push(Message::user("one".to_string()));
        t.scroll_down(10);
        assert_eq!(t.scroll, 0);
    }

    #[test]
    fn scroll_bottom_resets_offset() {
        let mut t = Transcript::new();
        t.push(Message::user("one".to_string()));
        t.scroll = 5;
        t.scroll_bottom();
        assert_eq!(t.scroll, 0);
    }

    #[test]
    fn push_caps_history() {
        let mut t = Transcript::new();
        for _ in 0..(MAX_MESSAGES + 10) {
            t.push(Message::user("x".to_string()));
        }
        assert_eq!(t.messages.len(), MAX_MESSAGES);
    }
}
