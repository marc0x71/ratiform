use ratatui::crossterm::event::KeyCode;

use crate::{
    field::{Field, FieldKind},
    widget::{
        check_box::handle_input_checkbox, select::handle_input_select,
        single_line::handle_input_singleline, text_area::handle_input_textarea,
    },
};

pub fn handle_input_field<T>(key_code: KeyCode, field: &mut Field<T>) {
    match field.kind {
        FieldKind::SingleLine(ref mut single_line) => {
            handle_input_singleline(key_code, single_line)
        }
        FieldKind::CheckBox(ref mut check_box) => handle_input_checkbox(key_code, check_box),
        FieldKind::Select(ref mut select) => handle_input_select(key_code, select),
        FieldKind::TextArea(ref mut text_area) => handle_input_textarea(key_code, text_area),
    }
    field.validate();
}

#[cfg(test)]
mod handle_input_field_tests {
    use super::*;
    use crate::{field::FieldOptions, validators, widget::single_line::SingleLineStatus};

    fn make_field(value: &str, required: Option<crate::field::Validator>) -> Field<i32> {
        Field {
            id: 1,
            kind: FieldKind::SingleLine(SingleLineStatus {
                label: "Test".to_owned(),
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

    #[test]
    fn handle_input_field_revalidates_when_a_keystroke_makes_the_value_invalid() {
        let mut field = make_field("A", Some(validators::required("Obbligatorio".to_owned())));

        // Backspace on a 1-character field with the cursor at the end
        // empties it, which should immediately turn the field invalid.
        handle_input_field(KeyCode::Backspace, &mut field);

        assert_eq!(field.error, Some("Obbligatorio".to_owned()));
    }

    #[test]
    fn handle_input_field_revalidates_when_a_keystroke_makes_the_value_valid_again() {
        let mut field = make_field("", Some(validators::required("Obbligatorio".to_owned())));

        handle_input_field(KeyCode::Char('A'), &mut field);

        assert_eq!(field.error, None);
    }
}
