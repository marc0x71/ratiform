use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::Style,
    text::Span,
    widgets::Widget,
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions},
    field_builder_common,
};

// BUILDER
/// Builder for a checkbox field, started with
/// [`FormBuilder::checkbox`](crate::builder::FormBuilder::checkbox).
/// Like the other field builders, it supports the common options
/// `required`, `optional`, `disabled`, `readonly`, `height`,
/// `validator`, and `normalizer`.
pub struct CheckboxBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) checked: bool,
    pub(crate) options: FieldOptions,
}

impl<T: PartialEq> CheckboxBuilder<T> {
    /// Sets whether the checkbox starts out checked.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Adds a validator that fails unless the checkbox is checked
    /// (`"true"`) — this actually enforces "must be checked",
    /// e.g. for a terms-and-conditions box.
    pub fn must_be_checked(mut self, message: String) -> Self {
        self.options.validator.push(Box::new(move |value: &str| {
            (value == "true")
                .then_some(())
                .ok_or_else(|| message.clone())
        }));
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        let initial_value = self.checked.to_string();
        self.form.push_field(Field {
            id: self.id,
            kind: FieldKind::CheckBox(CheckBoxStatus {
                label: self.label,
                checked: self.checked,
            }),
            options: self.options,
            error: None,
            initial_value,
        });

        self.form
    }
}
field_builder_common!(CheckboxBuilder<T>);

// STATUS
#[derive(Debug)]
pub struct CheckBoxStatus {
    pub(crate) label: String,
    pub(crate) checked: bool,
}

impl CheckBoxStatus {
    pub(crate) fn get(&self) -> String {
        self.checked.to_string()
    }
    fn toggle(&mut self) {
        self.checked = !self.checked
    }

    pub(crate) fn set(&mut self, value: &str) {
        self.checked = value.to_lowercase().parse().unwrap_or_default();
    }

    pub(crate) fn get_ref(&self) -> Cow<'_, str> {
        Cow::Owned(self.checked.to_string())
    }
}

// EVENT
pub(crate) fn handle_input_checkbox(key_event: KeyEvent, check_box: &mut CheckBoxStatus) {
    if let KeyCode::Char(' ') = key_event.code
        && !key_event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        check_box.toggle();
    }
}

// RENDER
pub(crate) fn render_checkbox(
    area: Rect,
    buf: &mut Buffer,
    checkbox: &mut CheckBoxStatus,
    value_style: Style,
    _highlight_style: Style,
) -> Option<(u16, u16)> {
    let flag = if checkbox.checked { "[✓]" } else { "[ ]" };

    let value = Span::raw(flag).style(value_style);

    value.render(area, buf);

    None
}

#[cfg(test)]
mod checkbox_tests {
    use ratatui::crossterm::event::KeyModifiers;

    use super::*;

    fn make_checkbox(checked: bool) -> CheckBoxStatus {
        CheckBoxStatus {
            label: "Test".to_owned(),
            checked,
        }
    }

    #[test]
    fn set_with_an_invalid_string_resets_to_false_rather_than_keeping_the_old_value() {
        // bool::from_str fails on anything other than "true"/"false", and
        // .unwrap_or_default() falls back to `false` — it does NOT leave
        // the previous value untouched. Starting from `true` on purpose,
        // so a wrong implementation that just ignores bad input would fail
        // this test.
        let mut checkbox = make_checkbox(true);
        checkbox.set("yes");
        assert!(!checkbox.checked);
    }

    #[test]
    fn set_is_case_insensitive() {
        let mut checkbox = make_checkbox(true);
        checkbox.set("True");
        assert!(checkbox.checked);
    }

    #[test]
    fn space_toggles_the_checkbox() {
        let mut checkbox = make_checkbox(false);
        handle_input_checkbox(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            &mut checkbox,
        );
        assert!(checkbox.checked);
    }

    #[test]
    fn other_keys_do_not_toggle_the_checkbox() {
        let mut checkbox = make_checkbox(false);
        handle_input_checkbox(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            &mut checkbox,
        );
        handle_input_checkbox(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut checkbox,
        );
        handle_input_checkbox(
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            &mut checkbox,
        );
        assert!(!checkbox.checked);
    }

    #[test]
    fn ctrl_and_alt_do_not_toggle() {
        let mut checkbox = make_checkbox(false);

        handle_input_checkbox(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL),
            &mut checkbox,
        );
        handle_input_checkbox(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::ALT),
            &mut checkbox,
        );

        assert!(!checkbox.checked); // invariato, nessun toggle
    }

    #[test]
    fn shift_still_toggles() {
        let mut checkbox = make_checkbox(false);

        handle_input_checkbox(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::SHIFT),
            &mut checkbox,
        );

        assert!(checkbox.checked); // Shift non blocca, coerente con SingleLine
    }
}

#[cfg(test)]
mod builder_checkbox_tests {
    use crate::{FormResult, builder::FormBuilder};
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn enter() -> KeyEvent {
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    }

    #[test]
    fn must_be_checked_blocks_submit_when_unchecked() {
        let mut state = FormBuilder::new()
            .checkbox(1, "Accetto i termini")
            .must_be_checked("Devi accettare i termini".to_owned())
            .build()
            .unwrap();

        state.handle_input(enter());

        assert!(matches!(state.result(), FormResult::Working));
    }

    #[test]
    fn must_be_checked_allows_submit_when_checked_at_build_time() {
        let mut state = FormBuilder::new()
            .checkbox(1, "Accetto i termini")
            .checked(true)
            .must_be_checked("Devi accettare i termini".to_owned())
            .build()
            .unwrap();

        state.handle_input(enter());

        assert!(matches!(state.result(), FormResult::Submitted));
    }

    #[test]
    fn must_be_checked_allows_submit_after_checking_interactively() {
        let mut state = FormBuilder::new()
            .checkbox(1, "Accetto i termini")
            .must_be_checked("Devi accettare i termini".to_owned())
            .build()
            .unwrap();

        state.handle_input(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)); // spunta
        state.handle_input(enter());

        assert!(matches!(state.result(), FormResult::Submitted));
    }

    #[test]
    fn required_alone_is_still_a_no_op_on_a_checkbox() {
        // Non-regressione: required() da solo, senza must_be_checked(),
        // continua a non bloccare nulla su un Checkbox -- comportamento
        // invariato, e' quello che questa fix affianca, non sostituisce.
        let mut state = FormBuilder::new()
            .checkbox(1, "Accetto i termini")
            .required("Obbligatorio".to_owned())
            .build()
            .unwrap();

        state.handle_input(enter());

        assert!(matches!(state.result(), FormResult::Submitted));
    }
}
