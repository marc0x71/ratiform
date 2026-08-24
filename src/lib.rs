pub mod builder;
mod event;
mod field;
mod render;
pub mod style;
pub mod validators;
mod widget;

use std::marker::PhantomData;

use field::Field;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::{event::handle_input_field, style::FormStyle};

#[derive(Default, Clone, Copy)]
pub enum FormResult {
    Submitted,
    Cancelled,
    #[default]
    Working,
}

#[derive(Default)]
pub struct FormState<T> {
    fields: Vec<Field<T>>,
    focus: usize,
    cursor_position: Option<(u16, u16)>,
    result: FormResult,
}

impl<T: PartialEq> FormState<T> {
    pub(crate) fn new(mut fields: Vec<Field<T>>) -> Self {
        fields.iter_mut().for_each(|f| f.validate());
        Self {
            fields,
            focus: 0,
            cursor_position: None,
            result: FormResult::Working,
        }
    }

    pub(crate) fn max_label_length(&self) -> usize {
        self.fields
            .iter()
            .max_by_key(|c| c.label().chars().count())
            .map(|f| f.label().chars().count())
            .unwrap_or_default()
    }

    pub fn cursor_position(&self) -> Option<(u16, u16)> {
        self.cursor_position
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }
        match key_event.code {
            KeyCode::Enter if !self.has_errors() => self.result = FormResult::Submitted,
            KeyCode::Esc => self.result = FormResult::Cancelled,
            KeyCode::Tab if !self.fields.is_empty() => {
                self.focus = self.focus.wrapping_add(1) % self.fields.len();
            }
            KeyCode::BackTab if !self.fields.is_empty() => {
                self.focus = (self.focus + self.fields.len() - 1) % self.fields.len();
            }
            _ => {
                if !self.fields.is_empty()
                    && let Some(field) = self.fields.get_mut(self.focus)
                    && !field.options.disabled
                    && !field.options.readonly
                {
                    handle_input_field(key_event.code, field);
                }
            }
        }
    }

    pub fn result(&self) -> FormResult {
        self.result
    }

    fn has_errors(&self) -> bool {
        self.fields.iter().any(|f| f.has_error())
    }

    pub fn values(self) -> impl Iterator<Item = (T, String)> {
        self.fields.into_iter().map(|f| {
            let value = f.get();
            (f.id, value)
        })
    }

    pub fn value(&self, id: &T) -> Option<String> {
        self.fields.iter().find(|&f| f.id == *id).map(|f| f.get())
    }

    pub fn set_value(&mut self, id: &T, value: &str) {
        if let Some(f) = self.fields.iter_mut().find(|f| f.id == *id) {
            f.set(value);
        }
    }

    pub fn focused_field(&self) -> Option<&T> {
        self.fields.get(self.focus).map(|f| &f.id)
    }

    pub fn reset(&mut self) {
        self.fields.iter_mut().for_each(|f| f.reset());
    }

    pub fn is_dirty(&self) -> bool {
        self.fields.iter().any(|f| f.is_dirty())
    }

    pub fn is_field_dirty(&self, id: &T) -> Option<bool> {
        self.fields
            .iter()
            .find(|f| f.id == *id)
            .map(|f| f.is_dirty())
    }
}

pub struct Form<T> {
    style: FormStyle,
    _phantom: PhantomData<T>,
}

impl<T> Form<T> {
    pub fn with_style(style: FormStyle) -> Self {
        Self {
            style,
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for Form<T> {
    fn default() -> Self {
        Self {
            style: FormStyle::default(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod form_state_tests {
    use super::*;
    use crate::{
        field::{FieldKind, FieldOptions, Validator},
        validators,
        widget::single_line::SingleLineStatus,
    };
    use ratatui::crossterm::event::KeyModifiers;

    fn make_field(id: i32, label: &str, value: &str, required: Option<Validator>) -> Field<i32> {
        Field {
            id,
            kind: FieldKind::SingleLine(SingleLineStatus {
                label: label.to_owned(),
                value: value.to_owned(),
                position: value.chars().count() as u16,
                masked_with: None,
                placeholder: None,
            }),
            options: FieldOptions {
                required,
                disabled: false,
                readonly: false,
                height: 1,
                validator: vec![],
            },
            error: None,
            initial_value: value.to_owned(),
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    // ---------- Tab / BackTab ----------

    #[test]
    fn tab_moves_focus_forward_and_wraps_around() {
        let mut state = FormState::new(vec![
            make_field(1, "A", "", None),
            make_field(2, "B", "", None),
            make_field(3, "C", "", None),
        ]);
        assert_eq!(state.focused_field(), Some(&1));

        state.handle_input(key(KeyCode::Tab));
        assert_eq!(state.focused_field(), Some(&2));

        state.handle_input(key(KeyCode::Tab));
        assert_eq!(state.focused_field(), Some(&3));

        state.handle_input(key(KeyCode::Tab));
        assert_eq!(state.focused_field(), Some(&1));
    }

    #[test]
    fn back_tab_wraps_to_the_last_field_with_a_non_power_of_two_field_count() {
        // 3 fields on purpose: this is the exact case that was broken
        // before the fix (usize::MAX % 3 == 0, not len - 1).
        let mut state = FormState::new(vec![
            make_field(1, "A", "", None),
            make_field(2, "B", "", None),
            make_field(3, "C", "", None),
        ]);
        assert_eq!(state.focused_field(), Some(&1));

        state.handle_input(key(KeyCode::BackTab));
        assert_eq!(state.focused_field(), Some(&3));
    }

    // ---------- input handling ----------

    #[test]
    fn non_press_events_are_ignored() {
        let mut state = FormState::new(vec![make_field(1, "A", "", None)]);
        let mut release_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        release_event.kind = KeyEventKind::Release;

        state.handle_input(release_event);

        assert_eq!(state.value(&1), Some(String::new()));
    }

    #[test]
    fn a_disabled_field_does_not_receive_input() {
        let mut field = make_field(1, "A", "", None);
        field.options.disabled = true;
        let mut state = FormState::new(vec![field]);

        state.handle_input(key(KeyCode::Char('x')));

        assert_eq!(state.value(&1), Some(String::new()));
    }

    #[test]
    fn a_readonly_field_does_not_receive_input() {
        let mut field = make_field(1, "A", "", None);
        field.options.readonly = true;
        let mut state = FormState::new(vec![field]);

        state.handle_input(key(KeyCode::Char('x')));

        assert_eq!(state.value(&1), Some(String::new()));
    }

    // ---------- submit ----------

    #[test]
    fn enter_does_not_submit_while_a_field_is_invalid() {
        let mut state = FormState::new(vec![make_field(
            1,
            "A",
            "",
            Some(validators::required("Obbligatorio".to_owned())),
        )]);

        state.handle_input(key(KeyCode::Enter));

        assert!(matches!(state.result(), FormResult::Working));
    }

    #[test]
    fn enter_submits_once_every_field_is_valid() {
        let mut state = FormState::new(vec![make_field(
            1,
            "A",
            "Mario",
            Some(validators::required("Obbligatorio".to_owned())),
        )]);

        state.handle_input(key(KeyCode::Enter));

        assert!(matches!(state.result(), FormResult::Submitted));
    }

    // ---------- values / value ----------

    #[test]
    fn values_returns_every_field_with_its_current_value() {
        let state = FormState::new(vec![
            make_field(1, "Nome", "Mario", None),
            make_field(2, "Cognome", "Rossi", None),
        ]);

        let values: std::collections::HashMap<i32, String> = state.values().collect();

        assert_eq!(values.get(&1), Some(&"Mario".to_owned()));
        assert_eq!(values.get(&2), Some(&"Rossi".to_owned()));
    }

    #[test]
    fn value_returns_none_for_an_id_that_does_not_exist() {
        let state = FormState::new(vec![make_field(1, "A", "Mario", None)]);
        assert_eq!(state.value(&999), None);
    }

    // ---------- focused_field ----------

    #[test]
    fn focused_field_is_none_when_the_form_has_no_fields() {
        let state: FormState<i32> = FormState::new(vec![]);
        assert_eq!(state.focused_field(), None);
    }

    // ---------- is_dirty ----------

    #[test]
    fn is_dirty_is_true_if_at_least_one_field_changed() {
        // Uses .any() internally: this proves it's "at least one", not
        // "every field", which is the semantic a careless refactor
        // (swapping .any() for .all()) would silently break.
        let mut state = FormState::new(vec![
            make_field(1, "A", "Mario", None),
            make_field(2, "B", "Rossi", None),
        ]);
        assert!(!state.is_dirty());

        state.set_value(&2, "Bianchi");

        assert!(state.is_dirty());
    }

    // ---------- max_label_length ----------

    #[test]
    fn max_label_length_counts_characters_not_bytes() {
        // "Città" is 5 characters but 6 bytes in UTF-8 ('à' takes 2 bytes).
        // "Nome" is 4 characters. Byte-counting would wrongly report 6.
        let state = FormState::new(vec![
            make_field(1, "Nome", "", None),
            make_field(2, "Città", "", None),
        ]);
        assert_eq!(state.max_label_length(), 5);
    }

    // ---------- boot-time validation ----------

    #[test]
    fn new_validates_every_field_immediately() {
        let state = FormState::new(vec![make_field(
            1,
            "A",
            "",
            Some(validators::required("Obbligatorio".to_owned())),
        )]);

        // No handle_input call yet -- the error must already be there.
        assert!(state.has_errors());
    }
}
