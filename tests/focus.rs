mod common;

use common::key;
use ratatui::crossterm::event::KeyCode;
use ratiform::builder::FormBuilder;

const A: i32 = 1;
const B: i32 = 2;
const C: i32 = 3;

#[test]
fn first_field_gets_the_focus_when_it_can_receive_it() {
    // GUARD
    let state = FormBuilder::new()
        .single_line(A, "A")
        .single_line(B, "B")
        .build()
        .unwrap();
    assert_eq!(state.focused_field(), Some(&A));
}

#[test]
fn readonly_first_field_still_gets_the_focus() {
    // GUARD: unlike disabled(), readonly() fields *can* be focused.
    let state = FormBuilder::new()
        .single_line(A, "A")
        .readonly()
        .single_line(B, "B")
        .build()
        .unwrap();
    assert_eq!(state.focused_field(), Some(&A));
}

#[test]
fn a_disabled_first_field_is_skipped() {
    let state = FormBuilder::new()
        .single_line(A, "A")
        .disabled()
        .single_line(B, "B")
        .build()
        .unwrap();
    assert_eq!(state.focused_field(), Some(&B));
}

#[test]
fn a_hidden_first_field_is_skipped() {
    let state = FormBuilder::new()
        .single_line(A, "A")
        .hide()
        .single_line(B, "B")
        .build()
        .unwrap();
    assert_eq!(state.focused_field(), Some(&B));
}

#[test]
fn several_unfocusable_fields_in_a_row_are_all_skipped() {
    let state = FormBuilder::new()
        .single_line(A, "A")
        .disabled()
        .single_line(B, "B")
        .hide()
        .single_line(C, "C")
        .build()
        .unwrap();
    assert_eq!(state.focused_field(), Some(&C));
}

#[test]
fn typing_right_after_build_reaches_the_first_focusable_field() {
    // The symptom the user actually sees: keys typed into the "focused"
    // field vanish until they press Tab.
    let mut state = FormBuilder::new()
        .single_line(A, "A")
        .disabled()
        .single_line(B, "B")
        .build()
        .unwrap();
    state.handle_input(key(KeyCode::Char('x')));
    assert_eq!(state.value(&B), Some("x".to_owned()));
    assert_eq!(state.value(&A), Some(String::new()));
}

#[test]
fn reset_moves_the_focus_to_the_first_focusable_field() {
    let mut state = FormBuilder::new()
        .single_line(A, "A")
        .disabled()
        .single_line(B, "B")
        .single_line(C, "C")
        .build()
        .unwrap();
    state.handle_input(key(KeyCode::Tab));
    state.handle_input(key(KeyCode::Tab));
    state.reset();
    assert_eq!(state.focused_field(), Some(&B));
}

#[test]
fn reset_keeps_focusing_the_first_field_when_it_is_focusable() {
    // GUARD
    let mut state = FormBuilder::new()
        .single_line(A, "A")
        .single_line(B, "B")
        .build()
        .unwrap();
    state.handle_input(key(KeyCode::Tab));
    state.reset();
    assert_eq!(state.focused_field(), Some(&A));
}

#[test]
fn a_form_with_no_focusable_field_does_not_panic() {
    // GUARD. DECISION D1: what should focused_field() return here?
    // For now we only pin down "no panic".
    let mut state = FormBuilder::new()
        .single_line(A, "A")
        .disabled()
        .single_line(B, "B")
        .hide()
        .build()
        .unwrap();
    state.handle_input(key(KeyCode::Tab));
    state.handle_input(key(KeyCode::BackTab));
    state.handle_input(key(KeyCode::Char('x')));
    state.reset();
}
