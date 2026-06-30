use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::components::{
    input::Input,
    logo,
    status::{Kind as StatusKind, Status},
    transcript::{Message, Transcript},
};
use crate::config::Config;
use crate::nix::{self, Manifest};

const CTRL_C_WINDOW: Duration = Duration::from_millis(1_000);
const POLL_TIMEOUT: Duration = Duration::from_millis(100);

const MIN_INPUT_PANE: u16 = 3;
const DEFAULT_INPUT_PANE: u16 = MIN_INPUT_PANE;
const PANE_RESIZE_STEP: u16 = 1;
const SCROLL_STEP: usize = 1;
const MOUSE_SCROLL_STEP: usize = 3;

pub fn run() -> std::io::Result<()> {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("warning: failed to load config: {e}; using defaults");
        Config::default()
    });
    let keys = match config.keybindings.parse() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("warning: invalid keybinding in config: {e}; using defaults");
            Config::default()
                .keybindings
                .parse()
                .expect("defaults are valid")
        }
    };

    ratatui::run(|terminal| {
        let _mouse = enable_mouse()?;
        let mut input = Input::new(keys.editor.clone());
        let mut status = Status::new();
        let mut transcript = Transcript::new();
        let mut input_pane_height: u16 = DEFAULT_INPUT_PANE;
        let mut manifest: Option<Manifest> = None;
        let mut manifest_error: Option<String> = None;
        let mut show_manifest = false;

        let cwd = std::env::current_dir()?;
        let (introspect_tx, introspect_rx) = mpsc::channel::<Result<Manifest, _>>();
        thread::spawn(move || {
            let _ = introspect_tx.send(nix::manifest_for(&cwd));
        });

        loop {
            status.tick();

            if let Ok(result) = introspect_rx.try_recv() {
                match result {
                    Ok(m) => manifest = Some(m),
                    Err(e) => manifest_error = Some(format!("{e}")),
                }
            }

            terminal.draw(|frame| {
                let area = frame.area();

                let status_y = area.bottom().saturating_sub(1);
                let input_width = area.width.saturating_sub(2).max(20);
                let input_x = area.x + (area.width - input_width) / 2;
                let available_above = status_y.saturating_sub(area.y).max(3);

                let max_input_height = available_above.saturating_sub(1);
                if input_pane_height > max_input_height {
                    input_pane_height = max_input_height;
                }
                let needed = input.needed_height(input_width);
                let input_height = needed
                    .max(input_pane_height)
                    .clamp(MIN_INPUT_PANE, max_input_height);
                let input_y = status_y.saturating_sub(input_height);
                let input_area = Rect::new(input_x, input_y, input_width, input_height);
                let top_area =
                    Rect::new(area.x, area.y, area.width, input_y.saturating_sub(area.y));

                if transcript.is_empty() {
                    logo::render(frame, top_area);
                } else {
                    transcript.render(frame, top_area);
                }
                input.render(frame, input_area);

                let status_area = Rect::new(input_x, status_y, input_width, 1);
                status.render(frame, status_area);

                if show_manifest {
                    render_manifest_overlay(
                        frame,
                        area,
                        manifest.as_ref(),
                        manifest_error.as_deref(),
                    );
                }
            })?;

            if !event::poll(POLL_TIMEOUT)? {
                continue;
            }
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if handle_key(
                        &key,
                        &keys.app,
                        &mut input,
                        &mut status,
                        &mut transcript,
                        &mut input_pane_height,
                        &mut show_manifest,
                    ) {
                        break;
                    }
                    if let Some(text) = input.take_submit() {
                        transcript.push(Message::user(text));
                    }
                }
                Event::Mouse(mouse) => {
                    if show_manifest {
                        continue;
                    }
                    if let Ok(size) = terminal.size() {
                        let area = Rect::new(0, 0, size.width, size.height);
                        handle_mouse(&mouse, area, &input, input_pane_height, &mut transcript);
                    }
                }
                _ => {}
            }
        }
        Ok::<(), std::io::Error>(())
    })?;

    Ok(())
}

fn handle_key(
    key: &crossterm::event::KeyEvent,
    app: &crate::config::ParsedAppKeybindings,
    input: &mut Input,
    status: &mut Status,
    transcript: &mut Transcript,
    input_pane_height: &mut u16,
    show_manifest: &mut bool,
) -> bool {
    if app.quit.matches(key) {
        return true;
    }
    if app.quit_force.matches(key) {
        if status.has(StatusKind::QuitNag) {
            return true;
        }
        status.set(
            StatusKind::QuitNag,
            "Press Ctrl+C again to close".to_string(),
            Some(CTRL_C_WINDOW),
        );
        return false;
    }

    if app.show_manifest.matches(key) {
        *show_manifest = !*show_manifest;
        return false;
    }
    if *show_manifest {
        return false;
    }

    if app.pane_grow.matches(key) {
        *input_pane_height = input_pane_height
            .saturating_add(PANE_RESIZE_STEP)
            .max(MIN_INPUT_PANE);
        return false;
    }
    if app.pane_shrink.matches(key) {
        *input_pane_height = input_pane_height
            .saturating_sub(PANE_RESIZE_STEP)
            .max(MIN_INPUT_PANE);
        return false;
    }

    if app.scroll_up.matches(key) {
        transcript.scroll_up(SCROLL_STEP);
        return false;
    }
    if app.scroll_down.matches(key) {
        transcript.scroll_down(SCROLL_STEP);
        return false;
    }
    if app.scroll_page_up.matches(key) {
        transcript.scroll_up(transcript.visible_lines().max(1));
        return false;
    }
    if app.scroll_page_down.matches(key) {
        transcript.scroll_down(transcript.visible_lines().max(1));
        return false;
    }
    if app.scroll_top.matches(key) {
        transcript.scroll_top();
        return false;
    }
    if app.scroll_bottom.matches(key) {
        transcript.scroll_bottom();
        return false;
    }

    if input.handle_key(key) {
        status.clear(StatusKind::QuitNag);
    }
    false
}

fn handle_mouse(
    mouse: &crossterm::event::MouseEvent,
    area: Rect,
    input: &Input,
    input_pane_height: u16,
    transcript: &mut Transcript,
) {
    let status_y = area.bottom().saturating_sub(1);
    let input_width = area.width.saturating_sub(2).max(20);
    let available_above = status_y.saturating_sub(area.y).max(3);
    let max_input_height = available_above.saturating_sub(1);
    let needed = input.needed_height(input_width);
    let input_height = needed
        .max(input_pane_height)
        .clamp(MIN_INPUT_PANE, max_input_height);
    let input_y = status_y.saturating_sub(input_height);

    if mouse.row < input_y {
        match mouse.kind {
            MouseEventKind::ScrollUp => transcript.scroll_up(MOUSE_SCROLL_STEP),
            MouseEventKind::ScrollDown => transcript.scroll_down(MOUSE_SCROLL_STEP),
            _ => {}
        }
    }
}

fn render_manifest_overlay(
    frame: &mut ratatui::Frame,
    area: Rect,
    manifest: Option<&Manifest>,
    error: Option<&str>,
) {
    let content = match manifest {
        Some(m) => m.summarize(),
        None => match error {
            Some(e) => format!("nix introspection failed:\n{e}"),
            None => "introspecting…".to_string(),
        },
    };

    let width = area.width.saturating_sub(4).clamp(40, 72);
    let inner_width = (width.saturating_sub(2)).max(1) as usize;
    let rows: Vec<String> = content
        .lines()
        .flat_map(|line| wrap_words(line, inner_width))
        .collect();
    let height = rows.len() as u16 + 2;
    let overlay = centered_rect(area, width, height);

    frame.render_widget(Clear, overlay);
    let block = Block::bordered().title("manifest");
    let lines: Vec<Line> = rows.iter().map(|row| Line::from(row.as_str())).collect();
    frame.render_widget(Paragraph::new(lines).block(block), overlay);
}

fn wrap_words(line: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut rows: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cur_len = 0usize;

    for word in line.split_whitespace() {
        let wlen = word.chars().count();
        if wlen > width {
            if !cur.is_empty() {
                rows.push(std::mem::take(&mut cur));
                cur_len = 0;
            }
            let mut buf = String::new();
            for ch in word.chars() {
                if buf.chars().count() == width {
                    rows.push(std::mem::take(&mut buf));
                }
                buf.push(ch);
            }
            if !buf.is_empty() {
                rows.push(buf);
            }
            continue;
        }
        if cur.is_empty() {
            cur.push_str(word);
            cur_len = wlen;
        } else if cur_len + 1 + wlen <= width {
            cur.push(' ');
            cur.push_str(word);
            cur_len += 1 + wlen;
        } else {
            rows.push(std::mem::take(&mut cur));
            cur.push_str(word);
            cur_len = wlen;
        }
    }

    if !cur.is_empty() || rows.is_empty() {
        rows.push(cur);
    }
    rows
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [col] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    col
}

struct MouseGuard;

impl Drop for MouseGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
    }
}

fn enable_mouse() -> std::io::Result<MouseGuard> {
    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
    Ok(MouseGuard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_words_breaks_on_spaces() {
        let rows = wrap_words("DevShell: rust-default-1.96.0, openssl-3.6.2", 20);
        assert!(rows.iter().all(|r| r.chars().count() <= 20));
        assert!(rows.contains(&"rust-default-1.96.0,".to_string()));
    }

    #[test]
    fn wrap_words_splits_long_token() {
        let rows = wrap_words("abcdefabcdefabcdef", 5);
        assert_eq!(rows, vec!["abcde", "fabcd", "efabc", "def"]);
    }

    #[test]
    fn wrap_words_empty_yields_one_row() {
        assert_eq!(wrap_words("", 10), vec![String::new()]);
    }
}
