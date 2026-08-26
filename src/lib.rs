//! A small, composable, stateful form widget for [Ratatui](https://github.com/ratatui/ratatui).
//!
//! Build forms with a typed field identity, keep the state in your own
//! application, and render them like any other Ratatui widget. Start with
//! [`builder::FormBuilder`] to build a form; [`FormState`] is what you keep
//! around and drive once it's built, and [`Form`] is the widget you render
//! it with.
//!
//! For the full story — design philosophy, a walkthrough, theming,
//! built-in validators — see the
//! [project README](https://github.com/marc0x71/ratiform).

pub mod builder;
mod event;
mod field;
mod render;
pub mod style;
pub mod validators;
mod widget;

use std::marker::PhantomData;

use field::Field;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{event::handle_input_field, style::FormStyle};

pub use field::Validator;
pub use widget::{
    check_box::CheckboxBuilder, select::SelectBuilder, single_line::SingleLineBuilder,
};

/// The form's current state, returned by [`FormState::result`].
#[derive(Default, Clone, Copy)]
pub enum FormResult {
    /// The user pressed `Enter` while every field was valid.
    Submitted,
    /// The user pressed `Esc`.
    Cancelled,
    /// The form is still being filled in — the default state.
    #[default]
    Working,
}

/// The live state of a form: which field has focus, the current value and
/// validation state of each field, and whether the form has been submitted
/// or cancelled. Built with [`builder::FormBuilder`], owned by your
/// application for as long as the form is active, and driven by feeding it
/// key events through [`handle_input`](FormState::handle_input).
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

    /// The absolute screen position the text cursor should be drawn at,
    /// if the currently focused field has a cursor at all (a `Select` or
    /// `Checkbox` never does). Pass this straight to Ratatui's
    /// `Frame::set_cursor_position`.
    pub fn cursor_position(&self) -> Option<(u16, u16)> {
        self.cursor_position
    }

    /// Feeds one key event to the form. Only [`KeyEventKind::Press`]
    /// events are handled; anything else (e.g. `Release`, reported by some
    /// terminals) is ignored, to avoid acting on the same key twice.
    ///
    /// A handful of keys are handled globally, regardless of which field
    /// has focus:
    ///
    /// | Key | Effect |
    /// | --- | --- |
    /// | `Tab` / `BackTab` | Move focus to the next / previous field, wrapping around |
    /// | `Enter` | Submit the form, unless some field is currently invalid |
    /// | `Esc` | Cancel the form |
    ///
    /// Every other key is routed to the focused field — unless it's
    /// `disabled()`/`readonly()`, in which case the key is dropped and
    /// nothing happens.
    pub fn handle_input(&mut self, key_event: KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::CONTROL, KeyCode::Enter) if !self.has_errors() => {
                self.result = FormResult::Submitted
            }
            (_, KeyCode::Enter)
                if !self.has_errors() && !self.focused_handle_key(KeyCode::Enter) =>
            {
                self.result = FormResult::Submitted
            }
            (_, KeyCode::Esc) => self.result = FormResult::Cancelled,
            (_, KeyCode::Tab) if !self.fields.is_empty() => {
                self.focus = self.focus.wrapping_add(1) % self.fields.len();
            }
            (_, KeyCode::BackTab) if !self.fields.is_empty() => {
                self.focus = (self.focus + self.fields.len() - 1) % self.fields.len();
            }
            (_, _) => {
                if !self.fields.is_empty()
                    && let Some(field) = self.fields.get_mut(self.focus)
                    && !field.options.disabled
                    && !field.options.readonly
                {
                    handle_input_field(key_event, field);
                }
            }
        }
    }

    /// The form's current state — see [`FormResult`].
    pub fn result(&self) -> FormResult {
        self.result
    }

    fn has_errors(&self) -> bool {
        self.fields.iter().any(|f| f.has_error())
    }

    /// Consumes the form and returns an iterator of `(id, value)` pairs,
    /// one per field — the usual way to collect the results after the form
    /// has been [`Submitted`](FormResult::Submitted) or
    /// [`Cancelled`](FormResult::Cancelled).
    pub fn values(self) -> impl Iterator<Item = (T, String)> {
        self.fields.into_iter().map(|f| {
            let value = f.get();
            (f.id, value)
        })
    }

    /// The current value of the field with the given id, or `None` if no
    /// field has it. Unlike [`values`](FormState::values), this can be
    /// called at any point while the form is still active.
    pub fn value(&self, id: &T) -> Option<String> {
        self.fields.iter().find(|&f| f.id == *id).map(|f| f.get())
    }

    /// Overwrites the value of the field with the given id, if one exists.
    /// Triggers validation immediately, exactly as if the user had typed
    /// it, so the field's error state is up to date before the next
    /// render. Does nothing if no field has that id.
    pub fn set_value(&mut self, id: &T, value: &str) {
        if let Some(f) = self.fields.iter_mut().find(|f| f.id == *id) {
            f.set(value);
        }
    }

    /// The id of the field that currently has focus, or `None` only if the
    /// form has no fields at all.
    pub fn focused_field(&self) -> Option<&T> {
        self.fields.get(self.focus).map(|f| &f.id)
    }

    /// Restores every field to the value it had when the form was built,
    /// and re-validates each one against the restored value.
    pub fn reset(&mut self) {
        self.fields.iter_mut().for_each(|f| f.reset());
    }

    /// Whether the value of the field with the given id has changed since
    /// the form was built, or `None` if no field has that id.
    pub fn is_dirty(&self) -> bool {
        self.fields.iter().any(|f| f.is_dirty())
    }

    pub fn is_field_dirty(&self, id: &T) -> Option<bool> {
        self.fields
            .iter()
            .find(|f| f.id == *id)
            .map(|f| f.is_dirty())
    }

    fn focused_handle_key(&self, code: KeyCode) -> bool {
        self.fields
            .get(self.focus)
            .map(|f| f.special_key_handled().contains(&code))
            .unwrap_or(false)
    }
}

/// The widget that renders a [`FormState`]. Stateless and cheap to
/// construct — build one fresh on every frame, exactly like any other
/// Ratatui widget, and pass it to `Frame::render_stateful_widget` together
/// with the `FormState` you want it to draw.
pub struct Form<T> {
    style: FormStyle,
    _phantom: PhantomData<T>,
}

impl<T> Form<T> {
    /// Renders with a custom [`FormStyle`] instead of the built-in theme.
    /// See the project README's Theming section for a full example.
    pub fn with_style(style: FormStyle) -> Self {
        Self {
            style,
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for Form<T> {
    /// Renders with the built-in theme. Equivalent to
    /// `Form::with_style(FormStyle::default())`.
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
