use git_version::git_version;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

const GIT_VERSION: &str = git_version!();

const LINE_1: &str = r#"▄█████▄ ▐██▌ ██▄ ▄██ ██   ██"#;
const LINE_2: &str = r#"██   ██  ██   ▀███▀  ██▄ ▄██"#;
const LINE_3: &str = r#"██   ██  ██   ▄███▄   ▀███▀ "#;
const LINE_4: &str = r#"██   ██ ▐██▌ ██▀ ▀██   ▐█▌  "#;

fn text<'a>() -> Text<'a> {
    Text::from(vec![
        Line::from(LINE_1.fg(Color::Cyan)),
        Line::from(LINE_2.fg(Color::Blue)),
        Line::from(LINE_3.fg(Color::Magenta)),
        Line::from(vec![
            LINE_4.fg(Color::Green),
            Span::raw("  "),
            format!("({})", GIT_VERSION).fg(Color::Yellow),
        ]),
    ])
}

pub fn render(frame: &mut Frame, area: Rect) {
    let logo = text();

    let [block] = Layout::vertical([Constraint::Length(logo.height() as u16)])
        .flex(Flex::Center)
        .areas(area);
    let [block] = Layout::horizontal([Constraint::Length(logo.width() as u16)])
        .flex(Flex::Center)
        .areas(block);

    frame.render_widget(Paragraph::new(logo).alignment(Alignment::Left), block);
}