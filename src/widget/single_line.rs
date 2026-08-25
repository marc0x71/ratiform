use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions},
    field_builder_common,
};

// BUILDER
/// Builder for a single-line text field, started with
/// [`FormBuilder::single_line`](crate::builder::FormBuilder::single_line).
/// For the options shared with every other field kind (`required`,
/// `optional`, `disabled`, `readonly`, `height`, `validator`), see
/// [`field_builder_common`](crate::field_builder_common).
pub struct SingleLineBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) options: FieldOptions,
    pub(crate) masked_with: Option<char>,
    pub(crate) placeholder: Option<String>,
}

impl<T: PartialEq> SingleLineBuilder<T> {
    /// Sets the field's initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Masks the field's displayed content with the given character —
    /// useful for password-style fields. Only affects rendering: the real
    /// value, validation, and everything read back through
    /// `value()`/`values()` all still see what the user actually typed.
    pub fn masked_with(mut self, c: char) -> Self {
        self.masked_with = Some(c);
        self
    }

    /// Equivalent to `masked_with('*')`.
    pub fn masked(mut self) -> Self {
        self.masked_with = Some('*');
        self
    }

    /// Sets a hint shown while the field is empty, styled with
    /// `FormStyle::placeholder`. Disappears as soon as the user types
    /// anything, and has no effect on validation: a required field
    /// showing a placeholder is still considered empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        let position = self.value.len() as u16;
        let initial_value = self.value.clone();
        self.form.fields.push(Field {
            id: self.id,
            kind: FieldKind::SingleLine(SingleLineStatus {
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
field_builder_common!(SingleLineBuilder<T>);

// STATUS
pub struct SingleLineStatus {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) position: u16,
    pub(crate) masked_with: Option<char>,
    pub(crate) placeholder: Option<String>,
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
    placeholder_style: Style,
) -> Option<(u16, u16)> {
    let mut style = value_style;

    let mut display = masked_display(singleline.value.as_str(), singleline.masked_with);
    if let Some(placeholder) = singleline.placeholder.as_ref()
        && display.is_empty()
    {
        display = Cow::Borrowed(placeholder);
        style = placeholder_style;
    }

    let value = Paragraph::new(display)
        .style(style)
        .block(Block::default().style(highlight_style));

    value.render(area, buf);

    Some((area.x + singleline.position, area.y))
}

fn masked_display(value: &str, mask: Option<char>) -> Cow<'_, str> {
    if let Some(c) = mask {
        let s = value.chars().map(|_| c).collect();
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

#[cfg(test)]
mod editing_tests {
    use super::*;

    fn make_status(value: &str, position: u16) -> SingleLineStatus {
        SingleLineStatus {
            label: "Test".to_owned(),
            value: value.to_owned(),
            position,
            masked_with: None,
            placeholder: None,
        }
    }

    #[test]
    fn insert_handles_unicode_content_correctly() {
        // "città" = c-i-t-t-à, position 3 sits right before the second 't'.
        let mut status = make_status("città", 3);
        handle_input_singleline(KeyCode::Char('X'), &mut status);

        assert_eq!(status.value, "citXtà");
        assert_eq!(status.position, 4);
    }

    #[test]
    fn backspace_removes_the_character_before_the_cursor() {
        // Cursor at the end: backspace must remove the 'à' (multi-byte)
        // as a whole character, not split it or remove the wrong byte.
        let mut status = make_status("città", 5);
        handle_input_singleline(KeyCode::Backspace, &mut status);

        assert_eq!(status.value, "citt");
        assert_eq!(status.position, 4);
    }

    #[test]
    fn backspace_at_the_start_does_not_panic_or_change_anything() {
        let mut status = make_status("città", 0);
        handle_input_singleline(KeyCode::Backspace, &mut status);

        assert_eq!(status.value, "città");
        assert_eq!(status.position, 0);
    }

    #[test]
    fn delete_removes_the_character_at_the_cursor() {
        // Position 4 sits right before the 'à'. Delete must remove it.
        let mut status = make_status("città", 4);
        handle_input_singleline(KeyCode::Delete, &mut status);

        assert_eq!(status.value, "citt");
        assert_eq!(status.position, 4);
    }

    #[test]
    fn delete_at_the_end_of_the_value_does_not_remove_anything() {
        let mut status = make_status("citt", 4);
        handle_input_singleline(KeyCode::Delete, &mut status);

        assert_eq!(status.value, "citt");
    }

    #[test]
    fn delete_on_an_empty_value_does_not_panic() {
        let mut status = make_status("", 0);
        handle_input_singleline(KeyCode::Delete, &mut status);

        assert_eq!(status.value, "");
        assert_eq!(status.position, 0);
    }

    #[test]
    fn end_moves_the_cursor_to_the_character_count_not_the_byte_length() {
        // "città" = 5 characters, 6 bytes: end() must stop at 5, not 6.
        // A direct regression test for the very first bug found in this project.
        let mut status = make_status("città", 0);
        handle_input_singleline(KeyCode::End, &mut status);

        assert_eq!(status.position, 5);
    }

    #[test]
    fn left_does_not_go_below_zero() {
        let mut status = make_status("ciao", 0);
        handle_input_singleline(KeyCode::Left, &mut status);

        assert_eq!(status.position, 0);
    }

    #[test]
    fn right_does_not_go_past_the_end() {
        let mut status = make_status("ciao", 4);
        handle_input_singleline(KeyCode::Right, &mut status);

        assert_eq!(status.position, 4);
    }
}
