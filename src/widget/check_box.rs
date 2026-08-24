use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
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
pub struct CheckboxBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) checked: bool,
    pub(crate) options: FieldOptions,
}

impl<T: PartialEq> CheckboxBuilder<T> {
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        self.form.fields.push(Field {
            id: self.id,
            kind: FieldKind::CheckBox(CheckBoxStatus {
                label: self.label,
                checked: self.checked,
            }),
            options: self.options,
            error: None,
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
        self.checked = value.parse().unwrap_or_default();
    }
}

// EVENT
pub(crate) fn handle_input_checkbox(key_code: KeyCode, check_box: &mut CheckBoxStatus) {
    if let KeyCode::Char(' ') = key_code {
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
