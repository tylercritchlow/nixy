use std::time::{Duration, Instant};

use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Agent,
    Error,
    QuitNag,
}

impl Kind {
    fn priority(self) -> u8 {
        match self {
            Kind::QuitNag => 100,
            Kind::Error => 50,
            Kind::Agent => 10,
        }
    }

    fn color(self) -> Color {
        match self {
            Kind::QuitNag => Color::Yellow,
            Kind::Error => Color::Red,
            Kind::Agent => Color::Cyan,
        }
    }
}

struct Entry {
    kind: Kind,
    text: String,
    priority: u8,
    expires_at: Option<Instant>,
    seq: u64,
}

pub struct Status {
    entries: Vec<Entry>,
    next_seq: u64,
}

impl Status {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_seq: 0,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.entries.retain(|e| e.expires_at.is_none_or(|t| t > now));
    }

    pub fn set(&mut self, kind: Kind, text: String, ttl: Option<Duration>) {
        let expires_at = ttl.map(|d| Instant::now() + d);
        let seq = self.next_seq;
        self.next_seq += 1;

        if let Some(entry) = self.entries.iter_mut().find(|e| e.kind == kind) {
            entry.text = text;
            entry.expires_at = expires_at;
            entry.seq = seq;
        } else {
            self.entries.push(Entry {
                kind,
                text,
                priority: kind.priority(),
                expires_at,
                seq,
            });
        }
    }

    pub fn clear(&mut self, kind: Kind) {
        self.entries.retain(|e| e.kind != kind);
    }

    pub fn has(&self, kind: Kind) -> bool {
        self.entries.iter().any(|e| e.kind == kind)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let Some(entry) = self
            .entries
            .iter()
            .max_by(|a, b| a.priority.cmp(&b.priority).then(a.seq.cmp(&b.seq)))
        else {
            return;
        };

        let text = truncate(entry.text.as_str(), area.width as usize);
        frame.render_widget(
            Paragraph::new(text.fg(entry.kind.color())).alignment(Alignment::Left),
            area,
        );
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(1);
    s.chars().take(keep).collect::<String>() + "…"
}