use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions, Requirement},
    field_builder_common,
};

// BUILDER
pub struct SingleLineBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) options: FieldOptions,
}

impl<T: PartialEq> SingleLineBuilder<T> {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        let position = self.value.len() as u16;
        self.form.fields.push(Field {
            id: self.id,
            kind: FieldKind::SingleLine(SingleLineStatus {
                label: self.label,
                value: self.value,
                position,
            }),
            options: self.options,
            error: None,
        });

        self.form
    }
}
field_builder_common!(SingleLineBuilder<T>);

// STATUS
pub struct SingleLineStatus {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) position: u16,
}

impl SingleLineStatus {
    pub(crate) fn get(&self) -> String {
        self.value.clone()
    }

    pub(crate) fn set(&mut self, value: &str) {
        self.value = value.to_string();
    }

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
        self.position = self.value.chars().count() as u16
    }
    fn insert(&mut self, c: char) {
        let byte_idx = self.byte_position(self.position, self.value.len());
        self.value.insert(byte_idx, c);
        self.position += 1;
    }
}

// EVENT
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

// RENDER
pub(crate) fn render_singleline(
    area: Rect,
    buf: &mut Buffer,
    singleline: &mut SingleLineStatus,
    value_style: Style,
    highlight_style: Style,
) -> Option<(u16, u16)> {
    let value = Paragraph::new(singleline.value.as_str())
        .style(value_style)
        .block(Block::default().style(highlight_style));

    value.render(area, buf);

    Some((area.x + singleline.position, area.y))
}
