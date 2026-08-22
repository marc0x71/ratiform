use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::field::{CheckBox, Field, FieldKind, Select, SingleLine};

pub fn handle_input_field(key_code: KeyCode, field: &mut Field) {
    match field.kind {
        FieldKind::SingleLine(ref mut single_line) => {
            handle_input_singleline(key_code, single_line)
        }
        FieldKind::CheckBox(ref mut check_box) => handle_input_checkbox(key_code, check_box),
        FieldKind::Select(ref mut select) => handle_input_select(key_code, select),
    }
}

pub fn handle_input_singleline(key_code: KeyCode, single_line: &mut SingleLine) {
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
pub fn handle_input_checkbox(key_code: KeyCode, check_box: &mut CheckBox) {
    if let KeyCode::Char(' ') = key_code {
        check_box.toggle();
    }
}
pub fn handle_input_select(key_code: KeyCode, select: &mut Select) {
    match key_code {
        KeyCode::Up => select.up(),
        KeyCode::Down => select.down(),
        KeyCode::Home => select.home(),
        KeyCode::End => select.end(),
        KeyCode::PageUp => select.page_up(),
        KeyCode::PageDown => select.page_down(),
        _ => {}
    }
}
