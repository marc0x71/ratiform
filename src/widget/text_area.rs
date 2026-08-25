#![allow(unused)]
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions},
    field_builder_common,
    widget::text_area,
};

// BUILDER
pub struct TextAreaBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) options: FieldOptions,
    pub(crate) masked_with: Option<char>,
    pub(crate) placeholder: Option<String>,
}

impl<T: PartialEq> TextAreaBuilder<T> {
    /// Sets the field's initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        let initial_value = self.value.to_string();
        let position = self.value.len() as u16;
        self.form.fields.push(Field {
            id: self.id,
            kind: FieldKind::TextArea(TextAreaStatus {
                label: self.label,
                value: self.value,
                position,
                masked_with: self.masked_with,
                placeholder: self.placeholder,
            }),
            options: self.options,
            error: None,
            initial_value,
        });

        self.form
    }
}
field_builder_common!(TextAreaBuilder<T>);

// STATUS
pub struct TextAreaStatus {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) position: u16,
    pub(crate) masked_with: Option<char>,
    pub(crate) placeholder: Option<String>,
}

impl TextAreaStatus {
    pub(crate) fn get(&self) -> String {
        self.value.clone()
    }

    pub(crate) fn set(&mut self, value: &str) {
        self.value = value.to_owned()
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
pub(crate) fn handle_input_textarea(key_code: KeyCode, text_area: &mut TextAreaStatus) {
    match key_code {
        KeyCode::Backspace => text_area.backspace(),
        KeyCode::Left => text_area.left(),
        KeyCode::Right => text_area.right(),
        KeyCode::Home => text_area.home(),
        KeyCode::End => text_area.end(),
        KeyCode::Delete => text_area.delete(),
        KeyCode::Char(c) => text_area.insert(c),
        _ => {}
    }
}

// RENDER
pub(crate) fn render_textarea(
    area: Rect,
    buf: &mut Buffer,
    text_area: &mut TextAreaStatus,
    value_style: Style,
    highlight_style: Style,
    placeholder_style: Style,
) -> Option<(u16, u16)> {
    let lines = wrap_text(&text_area.value, area.width as usize);

    let display = lines
        .iter()
        .map(|(_, line)| Line::from(line.clone()))
        .collect::<Vec<_>>();

    let value = Paragraph::new(display)
        .style(value_style)
        .block(Block::default().style(highlight_style));

    value.render(area, buf);

    let (x, y) = calculate_position(text_area, &lines);
    Some((area.x + x, area.y + y - 1))
}

fn calculate_position(text_area: &TextAreaStatus, lines: &[(usize, String)]) -> (u16, u16) {
    let mut y: u16 = 0;
    let mut begin: u16 = 0;

    for (start, _) in lines {
        if *start as u16 > text_area.position {
            break;
        }
        begin = *start as u16;
        y += 1;
    }
    let x = text_area.position - begin;
    (x, y)
}

fn wrap_text(text: &str, width: usize) -> Vec<(usize, String)> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut pos = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        // too long word
        if word_len >= width {
            if !current.is_empty() {
                let new_pos = pos + current.chars().count() + 1;
                lines.push((0, std::mem::take(&mut current)));
                pos = new_pos;
            }

            let chars: Vec<char> = word.chars().collect();
            for block in chars.chunks(width) {
                if block.len() < width {
                    current = block.iter().collect();
                    break;
                }
                let block: String = block.iter().collect();
                if !block.is_empty() {
                    // ma serve?!?!!?
                    let new_pos = pos + block.chars().count();
                    lines.push((pos, block));
                    pos = new_pos;
                }
            }
            continue;
        }

        // check length
        if current.len() + word_len + 1 > width {
            let new_pos = pos + current.chars().count() + 1;
            lines.push((pos, current));
            pos = new_pos;
            current = word.to_owned();
            continue;
        }

        // append
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        lines.push((pos, current));
    }
    lines
}

#[cfg(test)]
mod wrap_text_tests {
    use super::*;

    fn assert_at_positions(s: &str, checks: &[(usize, String)]) {
        for (pos, expected) in checks {
            let actual = s.get(*pos..*pos + expected.len());

            assert_eq!(
                actual,
                Some(expected.as_str()),
                "position {pos} expected {:?}, got {:?}",
                expected,
                actual
            );
        }
    }

    #[test]
    fn empty_string() {
        let result = wrap_text("", 10);
        assert_eq!(result, Vec::<(usize, String)>::new());
    }

    #[test]
    fn only_whitespace() {
        let result = wrap_text("     ", 10);
        assert_eq!(result, Vec::<(usize, String)>::new());
    }

    #[test]
    fn single_short_word() {
        let result = wrap_text("hello", 10);
        assert_eq!(result, vec![(0usize, "hello".to_owned())]);
    }

    #[test]
    fn simple_wrapping() {
        let s = "the quick brown fox jumps over";
        let result = wrap_text(s, 10);
        assert_at_positions(s, &result);
    }

    #[test]
    fn isolated_word_longer_than_width() {
        let s = "supercalifragilisticexpialidocious";
        let result = wrap_text(s, 10);
        assert_at_positions(s, &result);
    }

    #[test]
    fn long_word_surrounded_by_normal_words() {
        let s = "hi supercalifragilisticexpialidocious to all";
        let result = wrap_text(s, 10);
        assert_at_positions(s, &result);
    }
}
