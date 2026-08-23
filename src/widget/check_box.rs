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
    field::{Field, FieldKind, FieldOptions, Requirement},
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

impl<T> CheckboxBuilder<T> {
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
    disabled: bool,
) -> Option<(u16, u16)> {
    let flag = if checkbox.checked { "[✓]" } else { "[ ]" };

    let modifier = if disabled {
        Modifier::CROSSED_OUT
    } else {
        Modifier::default()
    };

    let style = if has_focus {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    let value = Span::raw(flag).style(style).add_modifier(modifier);

    value.render(area, buf);

    None
}
