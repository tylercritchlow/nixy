use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

use crate::components::{
    input::Input,
    logo,
    status::{Kind as StatusKind, Status},
};

const CTRL_C_WINDOW: Duration = Duration::from_millis(1_000);
const POLL_TIMEOUT: Duration = Duration::from_millis(100);

pub fn run() -> std::io::Result<()> {
    ratatui::run(|terminal| {
        let mut input = Input::new();
        let mut status = Status::new();

        loop {
            status.tick();

            terminal.draw(|frame| {
                let area = frame.area();

                let status_y = area.bottom().saturating_sub(1);
                let input_width = area.width.saturating_sub(2).max(20);
                let input_x = area.x + (area.width - input_width) / 2;
                let available_above = status_y.saturating_sub(area.y).max(3);
                let input_height = input.needed_height(input_width).clamp(3, available_above);
                let input_y = status_y.saturating_sub(input_height);
                let input_area = Rect::new(input_x, input_y, input_width, input_height);
                let logo_area =
                    Rect::new(area.x, area.y, area.width, input_y.saturating_sub(area.y));

                logo::render(frame, logo_area);
                input.render(frame, input_area);

                let status_area = Rect::new(input_x, status_y, input_width, 1);
                status.render(frame, status_area);
            })?;

            if !event::poll(POLL_TIMEOUT)? {
                continue;
            }
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            match (ctrl, key.code) {
                (true, KeyCode::Char('d')) => break,
                (true, KeyCode::Char('c')) => {
                    if status.has(StatusKind::QuitNag) {
                        break;
                    }
                    status.set(
                        StatusKind::QuitNag,
                        "Press Ctrl+C again to close".to_string(),
                        Some(CTRL_C_WINDOW),
                    );
                }
                _ => {
                    if input.handle_key(key) {
                        status.clear(StatusKind::QuitNag);
                    }
                }
            }
        }
        Ok::<(), std::io::Error>(())
    })?;

    Ok(())
}
