//! The default theme must always let the user see *which* option is chosen
//! and *where* the focus is, in every field state.
//!
//! These tests only look at the rendered buffer, so they don't depend on how
//! the fix is implemented (theme rules, render code, or both).

mod common;

use common::{key, render, row};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Style},
    widgets::StatefulWidget,
};
use ratiform::{
    Form, FormLayout, FormState,
    builder::FormBuilder,
    style::{FormStyle, Parts, States, Widgets},
};

const X: i32 = 1;
const OTHER: i32 = 2;
const OPTIONS: &[(&str, &str)] = &[("a", "Alpha"), ("b", "Beta"), ("c", "Gamma")];

/// Style of the first cell of `word`, wherever it appears in the buffer.
fn style_of(buf: &Buffer, word: &str) -> Style {
    for y in 0..buf.area.height {
        let line: String = (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect();
        if let Some(byte_x) = line.find(word) {
            let x = line[..byte_x].chars().count() as u16;
            return buf[(x, y)].style();
        }
    }
    panic!("{word:?} not found in the rendered buffer");
}

/// `Beta` (index 1) is the chosen option: it must not look like the others.
fn assert_beta_stands_out(buf: &Buffer) {
    let beta = style_of(buf, "Beta");
    assert_ne!(beta, style_of(buf, "Alpha"), "Beta looks like Alpha");
    assert_ne!(beta, style_of(buf, "Gamma"), "Beta looks like Gamma");
}

fn draw(state: &mut FormState<i32>) -> Buffer {
    render(state, FormLayout::Horizontal, 60, 10)
}

// --- Select: the chosen option ---------------------------------------------

#[test]
fn select_horizontal_shows_the_selection_with_focus() {
    // GUARD: already true today (ACTIVE is reversed while focused).
    let mut state = FormBuilder::new()
        .select(X, "X")
        .values_ref(OPTIONS)
        .selected(1)
        .horizontal()
        .single_line(OTHER, "Other")
        .build()
        .unwrap();
    assert_eq!(state.focused_field(), Some(&X));
    assert_beta_stands_out(&draw(&mut state));
}

#[test]
fn select_horizontal_shows_the_selection_without_focus() {
    let mut state = FormBuilder::new()
        .select(X, "X")
        .values_ref(OPTIONS)
        .selected(1)
        .horizontal()
        .single_line(OTHER, "Other")
        .build()
        .unwrap();
    state.handle_input(key(KeyCode::Tab));
    assert_eq!(state.focused_field(), Some(&OTHER));
    assert_beta_stands_out(&draw(&mut state));
}

#[test]
fn select_vertical_without_highlight_symbol_shows_the_selection_without_focus() {
    // With the default "> " symbol the selection is still readable; without
    // it, the style is the only cue left.
    let mut state = FormBuilder::new()
        .select(X, "X")
        .values_ref(OPTIONS)
        .selected(1)
        .highlight_symbol("")
        .height(3)
        .single_line(OTHER, "Other")
        .build()
        .unwrap();
    state.handle_input(key(KeyCode::Tab));
    assert_beta_stands_out(&draw(&mut state));
}

#[test]
fn select_horizontal_shows_the_selection_when_readonly() {
    let mut state = FormBuilder::new()
        .select(X, "X")
        .values_ref(OPTIONS)
        .selected(1)
        .horizontal()
        .readonly()
        .single_line(OTHER, "Other")
        .build()
        .unwrap();
    assert_beta_stands_out(&draw(&mut state));
}

#[test]
fn select_horizontal_shows_the_selection_when_disabled() {
    // A disabled field still has (and submits) a value: it must be readable.
    let mut state = FormBuilder::new()
        .select(X, "X")
        .values_ref(OPTIONS)
        .selected(1)
        .horizontal()
        .disabled()
        .single_line(OTHER, "Other")
        .build()
        .unwrap();
    assert_beta_stands_out(&draw(&mut state));
}

// --- MultiSelect and CheckBox: already readable through their markers -------

#[test]
fn multi_select_marks_checked_options_in_every_state() {
    // GUARD: the "✓" marker carries the selection, whatever the style.
    for state_name in ["normal", "readonly", "disabled"] {
        let builder = FormBuilder::new()
            .multi_select(X, "X")
            .values_ref(OPTIONS)
            .selected(&[1])
            .height(3);
        let builder = match state_name {
            "readonly" => builder.readonly(),
            "disabled" => builder.disabled(),
            _ => builder,
        };
        let mut state = builder.single_line(OTHER, "Other").build().unwrap();
        if state_name == "normal" {
            state.handle_input(key(KeyCode::Tab));
        }
        let buf = draw(&mut state);
        let beta_row = (0..buf.area.height)
            .map(|y| row(&buf, y))
            .find(|r| r.contains("Beta"))
            .unwrap();
        assert!(beta_row.contains('✓'), "{state_name}: {beta_row:?}");
    }
}

#[test]
fn checkbox_shows_its_value_in_every_state() {
    // GUARD
    for state_name in ["normal", "readonly", "disabled"] {
        let builder = FormBuilder::new().checkbox(X, "Toggle").checked(true);
        let builder = match state_name {
            "readonly" => builder.readonly(),
            "disabled" => builder.disabled(),
            _ => builder,
        };
        let mut state = builder.single_line(OTHER, "Other").build().unwrap();
        let buf = draw(&mut state);
        assert!(
            row(&buf, 0).contains("[✓]"),
            "{state_name}: {:?}",
            row(&buf, 0)
        );
    }
}

// --- TextArea: theme rules must target TEXT_AREA, not SINGLE_LINE ----------

fn render_with_style(state: &mut FormState<i32>, style: FormStyle) -> Buffer {
    let area = Rect::new(0, 0, 40, 4);
    let mut buf = Buffer::empty(area);
    Form::default()
        .with_style(style)
        .render(area, &mut buf, state);
    buf
}

#[test]
fn text_area_follows_rules_written_for_text_area() {
    let style = FormStyle::builder()
        .add(
            Widgets::TEXT_AREA,
            Parts::TEXT,
            States::ANY,
            Style::default().fg(Color::Red),
        )
        .build();
    let mut state = FormBuilder::new()
        .text_area(X, "Notes")
        .value("hello")
        .build()
        .unwrap();
    let buf = render_with_style(&mut state, style);
    assert_eq!(style_of(&buf, "hello").fg, Some(Color::Red));
}

#[test]
fn text_area_ignores_rules_written_only_for_single_line() {
    let style = FormStyle::builder()
        .add(
            Widgets::SINGLE_LINE,
            Parts::TEXT,
            States::ANY,
            Style::default().fg(Color::Red),
        )
        .build();
    let mut state = FormBuilder::new()
        .text_area(X, "Notes")
        .value("hello")
        .build()
        .unwrap();
    let buf = render_with_style(&mut state, style);
    assert_ne!(style_of(&buf, "hello").fg, Some(Color::Red));
}
