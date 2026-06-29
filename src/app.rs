use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

use crate::components::{
    input::Input,
    logo,
    status::{Kind as StatusKind, Status},
    transcript::{Message, Transcript},
};
use crate::config::Config;

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

        loop {
            status.tick();

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
                    ) {
                        break;
                    }
                    if let Some(text) = input.take_submit() {
                        transcript.push(Message::user(text));
                    }
                }
                Event::Mouse(mouse) => {
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
