use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
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
/// [`FormBuilder::checkbox`](crate::builder::FormBuilder::checkbox). For
/// the options shared with every other field kind, see
/// [`field_builder_common`](crate::field_builder_common).
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

    fn finish(mut self) -> FormBuilder<T> {
        let initial_value = self.checked.to_string();
        self.form.fields.push(Field {
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
}

// EVENT
pub(crate) fn handle_input_checkbox(key_event: KeyEvent, check_box: &mut CheckBoxStatus) {
    if let KeyCode::Char(' ') = key_event.code {
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
}
