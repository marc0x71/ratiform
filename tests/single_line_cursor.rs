mod common;

use common::{cell, key, render};
use ratatui::{buffer::Buffer, crossterm::event::KeyCode};
use ratiform::{FormLayout, builder::FormBuilder};

const W: u16 = 5;
const ALPHABET: &str = "abcdefghijkl";

/// Renders a one-field form with the value on its own row (Stacked), so the
/// value area starts at x = 0 and is exactly `W` cells wide.
fn cursor_for(value: &str) -> ((u16, u16), Buffer) {
    let mut state = FormBuilder::new()
        .single_line(1, "A")
        .value(value)
        .build()
        .unwrap();
    let buf = render(&mut state, FormLayout::Stacked, W, 4);
    (
        state.cursor_position().expect("a SingleLine has a cursor"),
        buf,
    )
}

#[test]
fn cursor_never_leaves_the_field_whatever_the_value_length() {
    for len in 0..=ALPHABET.len() {
        let ((x, _), _) = cursor_for(&ALPHABET[..len]);
        assert!(x < W, "len {len}: cursor at x={x}, field is {W} cells wide");
    }
}

#[test]
fn cursor_at_the_end_of_a_long_value_sits_on_the_last_column() {
    for len in (W as usize - 1)..=ALPHABET.len() {
        let ((x, y), buf) = cursor_for(&ALPHABET[..len]);
        assert_eq!(x, W - 1, "len {len}");
        // ...and the character just before the cursor is the last one typed.
        let last = ALPHABET[..len].chars().last().unwrap().to_string();
        assert_eq!(cell(&buf, x - 1, y), last, "len {len}");
    }
}

#[test]
fn cursor_in_the_middle_of_a_short_value_is_not_scrolled() {
    // GUARD
    let mut state = FormBuilder::new()
        .single_line(1, "A")
        .value("abcdefghij")
        .build()
        .unwrap();
    state.handle_input(key(KeyCode::Home));
    state.handle_input(key(KeyCode::Right));
    state.handle_input(key(KeyCode::Right));
    let buf = render(&mut state, FormLayout::Stacked, W, 4);
    let (x, y) = state.cursor_position().unwrap();
    assert_eq!(x, 2);
    assert_eq!(cell(&buf, 0, y), "a");
}
