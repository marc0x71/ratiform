use std::borrow::Cow;

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
    pub(crate) masked_with: Option<char>,
}

impl<T: PartialEq> SingleLineBuilder<T> {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn masked_with(mut self, c: char) -> Self {
        self.masked_with = Some(c);
        self
    }

    pub fn masked(mut self) -> Self {
        self.masked_with = Some('*');
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
                masked_with: self.masked_with,
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
    pub(crate) masked_with: Option<char>,
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
    let display = masked_display(singleline.value.as_str(), singleline.masked_with);
    let value = Paragraph::new(display)
        .style(value_style)
        .block(Block::default().style(highlight_style));

    value.render(area, buf);

    Some((area.x + singleline.position, area.y))
}

fn masked_display(value: &str, mask: Option<char>) -> Cow<'_, str> {
    if let Some(c) = mask {
        let s = value.chars().map(|v| c).collect();
        Cow::Owned(s)
    } else {
        Cow::Borrowed(value)
    }
}

#[cfg(test)]
mod masked_display_tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn no_mask_returns_the_value_unchanged() {
        let result = masked_display("ciao", None);
        assert_eq!(result, "ciao");
    }

    #[test]
    fn no_mask_does_not_allocate() {
        // Se in futuro qualcuno "semplifica" la funzione facendola sempre
        // allocare, questo test deve rompersi anche se il contenuto resta corretto.
        let result = masked_display("ciao", None);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn no_mask_on_empty_value_returns_empty_borrowed() {
        let result = masked_display("", None);
        assert_eq!(result, "");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn mask_replaces_every_character_with_the_mask_char() {
        let result = masked_display("ciao", Some('•'));
        assert_eq!(result, "••••");
    }

    #[test]
    fn mask_on_empty_value_returns_empty_owned() {
        let result = masked_display("", Some('•'));
        assert_eq!(result, "");
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn mask_counts_characters_not_bytes() {
        // "città" = 5 caratteri, 6 byte (la 'à' occupa 2 byte in UTF-8).
        // Il caso che frega chi usa .len() invece di .chars().count().
        let result = masked_display("città", Some('•'));
        assert_eq!(result, "•••••");
        assert_eq!(result.chars().count(), 5);
    }

    #[test]
    fn mask_counts_multibyte_grapheme_as_one_character() {
        // Un'emoji come "🎉" è un solo char logico ma 4 byte in UTF-8.
        let result = masked_display("🎉hi", Some('•'));
        assert_eq!(result, "•••");
    }

    #[test]
    fn a_different_mask_char_is_respected() {
        let result = masked_display("hi", Some('*'));
        assert_eq!(result, "**");
    }

    #[test]
    fn mask_char_itself_can_be_multibyte() {
        // Il carattere di mascheramento può a sua volta essere multi-byte:
        // non deve influire sul conteggio delle ripetizioni.
        let result = masked_display("hi", Some('★'));
        assert_eq!(result, "★★");
        assert_eq!(result.chars().count(), 2);
    }
}
