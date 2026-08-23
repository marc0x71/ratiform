use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::{
    field::{Field, FieldKind},
    widget::{
        check_box::handle_input_checkbox, select::handle_input_select,
        single_line::handle_input_singleline,
    },
};

pub fn handle_input_field(key_code: KeyCode, field: &mut Field) {
    match field.kind {
        FieldKind::SingleLine(ref mut single_line) => {
            handle_input_singleline(key_code, single_line)
        }
        FieldKind::CheckBox(ref mut check_box) => handle_input_checkbox(key_code, check_box),
        FieldKind::Select(ref mut select) => handle_input_select(key_code, select),
    }
}
