mod common;

use common::{render, row, words};
use ratatui::layout::Constraint;
use ratiform::{
    FormLayout, FormState,
    builder::FormBuilder,
    layout::custom::{CustomLayout, Object, ObjectKind},
    required_height,
};

fn form_with_error(message: &'static str) -> FormState<i32> {
    FormBuilder::new()
        .single_line(1, "X")
        .value("x")
        .validator(move |_: &str| Err(message.to_owned()))
        .build()
        .unwrap()
}

#[test]
fn a_wrapped_error_is_shown_in_full_at_the_required_height() {
    // GUARD: ordinary word wrapping already agrees with the renderer.
    let message = "Il codice fiscale inserito non è valido";
    for width in 12..=40u16 {
        let mut state = form_with_error(message);
        let h = required_height(&FormLayout::Horizontal, &state, width);
        let buf = render(&mut state, FormLayout::Horizontal, width, h);
        let shown = (1..h).map(|y| row(&buf, y)).collect::<Vec<_>>().join(" ");
        assert_eq!(words(&shown), words(message), "width {width}");
    }
}

#[test]
fn error_with_an_explicit_newline_uses_two_rows() {
    let mut state = form_with_error("uno\ndue");
    assert_eq!(required_height(&FormLayout::Horizontal, &state, 30), 3);
    let buf = render(&mut state, FormLayout::Horizontal, 30, 3);
    assert_eq!(row(&buf, 1), "uno");
    assert_eq!(row(&buf, 2), "due");
}

#[test]
fn error_with_an_explicit_newline_uses_two_rows_in_the_stacked_layout() {
    let mut state = form_with_error("uno\ndue");
    assert_eq!(required_height(&FormLayout::Stacked, &state, 30), 4);
    let buf = render(&mut state, FormLayout::Stacked, 30, 4);
    assert_eq!(row(&buf, 2), "uno");
    assert_eq!(row(&buf, 3), "due");
}

#[test]
fn error_with_an_explicit_newline_uses_two_rows_in_a_custom_layout() {
    let layout = || {
        FormLayout::Custom(CustomLayout::new(vec![
            vec![(Constraint::Fill(1), Some(Object::new(ObjectKind::Value, 1)))],
            vec![(Constraint::Fill(1), Some(Object::new(ObjectKind::Error, 1)))],
        ]))
    };
    let mut state = form_with_error("uno\ndue");
    assert_eq!(required_height(&layout(), &state, 30), 3);
    let buf = render(&mut state, layout(), 30, 3);
    assert_eq!(row(&buf, 1), "uno");
    assert_eq!(row(&buf, 2), "due");
}
