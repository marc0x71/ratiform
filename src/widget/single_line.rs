use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph, Widget},
};

pub struct SingleLineStatus {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) position: u16,
}

impl SingleLineStatus {
    fn byte_position(&self, position: u16, default: usize) -> usize {
        self.value
            .char_indices()
            .nth(position as usize)
            .map_or(default, |(i, _)| i)
    }
    fn delete(&mut self) {
        if self.value.is_empty() {
            return;
        }
        let byte_idx = self.byte_position(self.position, self.value.len());
        if byte_idx < self.value.len() {
            self.value.remove(byte_idx);
            self.position = self.position.min(self.value.chars().count() as u16)
        }
    }
    fn backspace(&mut self) {
        if self.position == 0 {
            return;
        }
        let byte_idx = self.byte_position(self.position - 1, 0);
        self.value.remove(byte_idx);
        self.position = self.position.saturating_sub(1)
    }
    fn left(&mut self) {
        self.position = self.position.saturating_sub(1)
    }
    fn right(&mut self) {
        self.position = (self.position + 1).min(self.value.chars().count() as u16)
    }
    fn home(&mut self) {
        self.position = 0
    }
    fn end(&mut self) {
        self.position = self.value.len() as u16
    }
    fn insert(&mut self, c: char) {
        let byte_idx = self.byte_position(self.position, self.value.len());
        self.value.insert(byte_idx, c);
        self.position += 1;
    }
}

pub(crate) fn handle_input_singleline(key_code: KeyCode, single_line: &mut SingleLineStatus) {
    match key_code {
        KeyCode::Backspace => single_line.backspace(),
        KeyCode::Left => single_line.left(),
        KeyCode::Right => single_line.right(),
        KeyCode::Home => single_line.home(),
        KeyCode::End => single_line.end(),
        KeyCode::Delete => single_line.delete(),
        KeyCode::Char(c) => single_line.insert(c),
        _ => {}
    }
}

pub(crate) fn render_singleline(
    area: Rect,
    buf: &mut Buffer,
    singleline: &mut SingleLineStatus,
    has_focus: bool,
) -> Option<(u16, u16)> {
    let style = if has_focus {
        Style::default().fg(Color::Black).bg(Color::Gray)
    } else {
        Style::default().fg(Color::Gray)
    };

    let value = Paragraph::new(singleline.value.as_str()).block(Block::default().style(style));

    value.render(area, buf);

    if has_focus {
        Some((area.x + singleline.position, area.y))
    } else {
        None
    }
}
