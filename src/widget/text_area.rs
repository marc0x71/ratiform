#![allow(unused)]
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, layout::Rect, style::Style};

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
    todo!()
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        dbg!(word);
        let word_len = word.chars().count();

        // too long word
        if word_len >= width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
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
                    lines.push(block);
                }
            }
            continue;
        }

        // check length
        if current.len() + word_len + 1 > width {
            lines.push(current);
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
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod wrap_text_tests {
    use super::*;

    #[test]
    fn empty_string() {
        let result = wrap_text("", 10);
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn only_whitespace() {
        let result = wrap_text("     ", 10);
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn single_short_word() {
        let result = wrap_text("hello", 10);
        assert_eq!(result, vec!["hello"]);
    }

    #[test]
    fn simple_wrapping() {
        let result = wrap_text("the quick brown fox jumps over", 10);
        assert_eq!(result, vec!["the quick", "brown fox", "jumps over"]);
    }

    #[test]
    fn isolated_word_longer_than_width() {
        let result = wrap_text("supercalifragilisticexpialidocious", 10);
        assert_eq!(
            result,
            vec!["supercalif", "ragilistic", "expialidoc", "ious"]
        );
    }

    #[test]
    fn long_word_surrounded_by_normal_words() {
        let result = wrap_text("hi supercalifragilisticexpialidocious to all", 10);
        assert_eq!(
            result,
            vec![
                "hi",
                "supercalif",
                "ragilistic",
                "expialidoc",
                "ious to",
                "all"
            ]
        );
    }
}
