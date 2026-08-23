use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Widget,
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions, Requirement},
    field_builder_common,
};

// BUILDER
pub struct CheckboxBuilder {
    pub(crate) form: FormBuilder,
    pub(crate) label: String,
    pub(crate) checked: bool,
    pub(crate) options: FieldOptions,
}

impl CheckboxBuilder {
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    fn finish(mut self) -> FormBuilder {
        self.form.fields.push(Field {
            kind: FieldKind::CheckBox(CheckBoxStatus {
                label: self.label,
                checked: self.checked,
            }),
            options: self.options,
        });

        self.form
    }
}
field_builder_common!(CheckboxBuilder);

// STATUS
pub struct CheckBoxStatus {
    pub(crate) label: String,
    pub(crate) checked: bool,
}

impl CheckBoxStatus {
    fn toggle(&mut self) {
        self.checked = !self.checked
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
    has_focus: bool,
) -> Option<(u16, u16)> {
    let flag = if checkbox.checked { "[✓]" } else { "[ ]" };

    let style = if has_focus {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    let value = Span::raw(flag).style(style);

    value.render(area, buf);

    None
}
