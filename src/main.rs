use color_eyre::Result;
use ratatui::prelude::Widget;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut text = String::new();
    let mut cursor = 0usize;

    terminal.show_cursor()?;

    loop {
        terminal.clear()?;

        terminal.draw(|frame| {
            let area = frame.area();
            let paragraph = Paragraph::new(text.as_str());
            paragraph.render(area, frame.buffer_mut());

            let mut row = 0usize;
            let mut col = 0usize;
            let mut count = 0usize;
            for ch in text.chars() {
                if count >= cursor {
                    break;
                }
                count += 1;
                if ch == '\n' {
                    row += 1;
                    col = 0;
                } else {
                    col += 1;
                    if col >= area.width as usize {
                        row += 1;
                        col = 0;
                    }
                }
            }

            let x = col.min(area.width.saturating_sub(1) as usize) as u16;
            let y = row.min(area.height.saturating_sub(1) as usize) as u16;
            frame.set_cursor_position((x, y));
        })?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Esc => break,
                KeyCode::Backspace => {
                    if cursor > 0 {
                        cursor -= 1;
                        text.remove(cursor);
                    }
                }
                KeyCode::Left => {
                    cursor = cursor.saturating_sub(1);
                }
                KeyCode::Right => {
                    cursor = (cursor + 1).min(text.len());
                }
                KeyCode::Enter => {
                    text.insert(cursor, '\n');
                    cursor += 1;
                }
                KeyCode::Char(c) => {
                    text.insert(cursor, c);
                    cursor += c.len_utf8();
                }
                _ => {}
            },
            _ => {}
        }
    }

    terminal.hide_cursor()?;
    Ok(())
}