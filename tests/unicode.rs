mod common;

use common::{cell, ctrl, key, render};
use ratatui::{buffer::Buffer, crossterm::event::KeyCode};
use ratiform::{FormLayout, FormState, builder::FormBuilder};

fn value_column(first: &str, second: &str) -> u16 {
    let mut state = FormBuilder::new()
        .single_line(1, first)
        .single_line(2, second)
        .build()
        .unwrap();
    render(&mut state, FormLayout::Horizontal, 40, 6);
    state.cursor_position().unwrap().0
}

fn masked(value: &str) -> FormState<i32> {
    FormBuilder::new()
        .single_line(1, "A")
        .masked()
        .value(value)
        .build()
        .unwrap()
}

fn single_line(value: &str) -> FormState<i32> {
    FormBuilder::new()
        .single_line(1, "A")
        .value(value)
        .build()
        .unwrap()
}

fn press(state: &mut FormState<i32>, keys: &[KeyCode]) {
    for k in keys {
        state.handle_input(key(*k));
    }
}

fn screen_x(state: &mut FormState<i32>, w: u16) -> u16 {
    render(state, FormLayout::Stacked, w, 4);
    state
        .cursor_position()
        .expect("a SingleLine has a cursor")
        .0
}

fn field(value: &str) -> FormState<i32> {
    FormBuilder::new()
        .single_line(1, "A")
        .value(value)
        .build()
        .unwrap()
}

fn drawn_with_cursor(value: &str, w: u16) -> (Buffer, u16, u16) {
    let mut state = field(value);
    let buf = render(&mut state, FormLayout::Stacked, w, 3);
    let (x, y) = state.cursor_position().unwrap();
    (buf, x, y)
}

#[test]
fn ascii_labels_are_unchanged() {
    assert_eq!(value_column("Nome", "Cognome"), 8);
}

#[test]
fn wide_characters_count_two_cells() {
    assert_eq!(value_column("名前", "abc"), 5);
}

#[test]
fn combining_marks_take_no_extra_cell() {
    assert_eq!(value_column("e\u{301}e\u{301}", "abc"), 4);
}

#[test]
fn typing_after_a_prefilled_wide_value_appends() {
    let mut state = single_line("界界");
    press(&mut state, &[KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界界x".to_owned()));
}

#[test]
fn left_moves_back_over_exactly_one_character() {
    let mut state = single_line("界界");
    press(&mut state, &[KeyCode::Left, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界x界".to_owned()));
}

#[test]
fn end_puts_the_cursor_after_the_last_character() {
    let mut state = single_line("界界");
    press(
        &mut state,
        &[KeyCode::Home, KeyCode::End, KeyCode::Char('x')],
    );
    assert_eq!(state.value(&1), Some("界界x".to_owned()));
}

#[test]
fn right_stops_after_the_last_character() {
    let mut state = single_line("界界");
    press(&mut state, &[KeyCode::Home]);
    press(&mut state, &[KeyCode::Right; 5]);
    press(&mut state, &[KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界界x".to_owned()));
}

#[test]
fn delete_removes_the_character_under_the_cursor() {
    let mut state = single_line("界界");
    press(&mut state, &[KeyCode::Home, KeyCode::Delete]);
    assert_eq!(state.value(&1), Some("界".to_owned()));
}

#[test]
fn backspace_removes_the_character_before_the_cursor() {
    let mut state = single_line("界界");
    press(&mut state, &[KeyCode::Backspace]);
    assert_eq!(state.value(&1), Some("界".to_owned()));
}

#[test]
fn set_value_keeps_the_cursor_inside_the_new_value() {
    let mut state = single_line("界界界");
    state.set_value(&1, "界");
    press(&mut state, &[KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界x".to_owned()));
}

#[test]
fn text_area_left_moves_back_over_exactly_one_character() {
    let mut state = FormBuilder::new()
        .text_area(1, "T")
        .value("界界")
        .height(3)
        .build()
        .unwrap();
    render(&mut state, FormLayout::Stacked, 20, 6);
    state.handle_input(ctrl(KeyCode::End));
    press(&mut state, &[KeyCode::Left, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界x界".to_owned()));
}

#[test]
fn left_after_end_lands_between_the_two_characters() {
    let mut state = single_line("界界");
    press(&mut state, &[KeyCode::Home, KeyCode::End, KeyCode::Left]);
    press(&mut state, &[KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界x界".to_owned()));
}

#[test]
fn left_after_overshooting_with_right_lands_between_the_two_characters() {
    let mut state = single_line("界界");
    press(&mut state, &[KeyCode::Home]);
    press(&mut state, &[KeyCode::Right; 5]);
    press(&mut state, &[KeyCode::Left, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界x界".to_owned()));
}

#[test]
fn wide_characters_advance_the_screen_cursor_by_two_cells() {
    assert_eq!(screen_x(&mut field("界界"), 10), 4);
}

#[test]
fn a_narrow_character_after_wide_ones_adds_one_cell() {
    assert_eq!(screen_x(&mut field("界a"), 10), 3);
}

#[test]
fn the_screen_cursor_stays_inside_the_field_with_wide_characters() {
    assert!(screen_x(&mut field("界界界界界界"), 5) < 5);
}

#[test]
fn the_public_cursor_position_is_in_cells_and_index_position_in_characters() {
    let mut state = field("界界");
    render(&mut state, FormLayout::Stacked, 10, 4);
    assert_eq!(state.single_line(&1).unwrap().index_position(), 2);
    assert_eq!(state.single_line(&1).unwrap().cursor_position(), 4);
}

#[test]
fn a_masked_field_measures_what_is_drawn_not_what_is_typed() {
    let mut state = FormBuilder::new()
        .single_line(1, "A")
        .masked()
        .value("界界")
        .build()
        .unwrap();
    assert_eq!(screen_x(&mut state, 10), 2);
}

#[test]
fn a_masked_field_with_ascii_is_unchanged() {
    let mut state = FormBuilder::new()
        .single_line(1, "A")
        .masked()
        .value("abc")
        .build()
        .unwrap();
    assert_eq!(screen_x(&mut state, 10), 3);
}

#[test]
fn masked_cursor_in_the_middle_follows_the_index() {
    let mut state = masked("abc");
    state.handle_input(key(KeyCode::Home));
    state.handle_input(key(KeyCode::Right));
    assert_eq!(screen_x(&mut state, 10), 1);
}

#[test]
fn masked_cursor_in_the_middle_follows_the_index_with_wide_characters() {
    let mut state = masked("界界界");
    state.handle_input(key(KeyCode::Home));
    state.handle_input(key(KeyCode::Right));
    assert_eq!(screen_x(&mut state, 10), 1);
}

#[test]
fn masked_end_counts_what_is_drawn() {
    let mut state = masked("界界");
    state.handle_input(key(KeyCode::Home));
    state.handle_input(key(KeyCode::End));
    assert_eq!(screen_x(&mut state, 10), 2);
}

#[test]
fn a_prefilled_value_with_filtered_out_characters_places_the_cursor_after_the_kept_ones() {
    let mut state = FormBuilder::new()
        .single_line(1, "A")
        .alphabet("0123456789")
        .value("1a2")
        .build()
        .unwrap();
    assert_eq!(screen_x(&mut state, 10), 2);
}

#[test]
fn a_masked_prefilled_value_with_filtered_out_characters_draws_only_the_kept_ones() {
    let mut state = FormBuilder::new()
        .single_line(1, "A")
        .masked()
        .alphabet("0123456789")
        .value("1a2")
        .build()
        .unwrap();
    assert_eq!(screen_x(&mut state, 10), 2);
}

#[test]
fn the_last_narrow_character_sits_right_before_the_cursor_after_wide_ones() {
    let (buf, x, y) = drawn_with_cursor("界界界界界a", 5);
    assert_eq!(cell(&buf, x - 1, y), "a");
}

#[test]
fn the_last_wide_character_ends_right_before_the_cursor() {
    let (buf, x, y) = drawn_with_cursor("界界界界界界", 5);
    assert_eq!(cell(&buf, x - 2, y), "界");
}

#[test]
fn a_wide_character_after_a_narrow_one_ends_right_before_the_cursor() {
    let (buf, x, y) = drawn_with_cursor("a界界界界界", 5);
    assert_eq!(cell(&buf, x - 2, y), "界");
}

#[test]
fn at_the_end_the_last_character_always_ends_right_before_the_cursor() {
    use ratatui::buffer::CellWidth;
    for w in 3..=8u16 {
        for value in [
            "界界界界界a",
            "a界界界界界",
            "界a界a界a界a",
            "abc界de界fg",
            "界界界界界界",
            "abcdefghij",
        ] {
            let (buf, x, y) = drawn_with_cursor(value, w);
            assert!(x < w, "{value:?} in {w} cells: cursor at {x}");
            let last = value.chars().last().unwrap().to_string();
            assert_eq!(
                cell(&buf, x - last.cell_width(), y),
                last,
                "{value:?} in {w} cells"
            );
        }
    }
}

fn text_area(value: &str) -> FormState<i32> {
    FormBuilder::new()
        .text_area(1, "T")
        .value(value)
        .height(3)
        .build()
        .unwrap()
}

#[test]
fn text_area_ascii_wrapping_is_unchanged() {
    let mut state = text_area("abcdef");
    let buf = render(&mut state, FormLayout::Stacked, 4, 6);
    assert_eq!(cell(&buf, 0, 1), "a");
    assert_eq!(cell(&buf, 3, 1), "d");
    assert_eq!(cell(&buf, 0, 2), "e");
    assert_eq!(cell(&buf, 1, 2), "f");
}

#[test]
fn text_area_wraps_when_the_cells_run_out_not_the_chars() {
    let mut state = text_area("界界界");
    let buf = render(&mut state, FormLayout::Stacked, 4, 6);
    assert_eq!(cell(&buf, 0, 1), "界");
    assert_eq!(cell(&buf, 2, 1), "界");
    assert_eq!(cell(&buf, 0, 2), "界");
}

#[test]
fn text_area_never_splits_a_wide_character_across_rows() {
    let mut state = text_area("界界");
    let buf = render(&mut state, FormLayout::Stacked, 3, 6);
    assert_eq!(cell(&buf, 0, 1), "界");
    assert_eq!(cell(&buf, 0, 2), "界");
}

#[test]
fn text_area_screen_cursor_column_is_measured_in_cells() {
    let mut state = text_area("界a");
    press(&mut state, &[KeyCode::Right]);
    assert_eq!(screen_x(&mut state, 10), 2);
}

#[test]
fn text_area_down_lands_on_the_first_character_of_the_row_below() {
    let mut state = text_area("界界界界");
    render(&mut state, FormLayout::Stacked, 4, 6);
    press(&mut state, &[KeyCode::Down, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界界x界界".to_owned()));
}

#[test]
fn text_area_end_goes_after_the_last_character_of_the_last_row() {
    let mut state = text_area("abcd");
    render(&mut state, FormLayout::Stacked, 20, 6);
    press(&mut state, &[KeyCode::End, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("abcdx".to_owned()));
}

#[test]
fn text_area_end_goes_after_the_last_wide_character_of_the_last_row() {
    let mut state = text_area("界界界界");
    render(&mut state, FormLayout::Stacked, 20, 6);
    press(&mut state, &[KeyCode::End, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界界界界x".to_owned()));
}

#[test]
fn text_area_end_goes_after_the_last_character_of_a_row_ended_by_a_newline() {
    let mut state = text_area("ab\ncd");
    render(&mut state, FormLayout::Stacked, 20, 6);
    press(&mut state, &[KeyCode::End, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("abx\ncd".to_owned()));
}

#[test]
fn text_area_end_on_a_wrapped_row_stays_on_that_row() {
    let mut state = text_area("abcdefgh");
    render(&mut state, FormLayout::Stacked, 4, 6);
    press(&mut state, &[KeyCode::End, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("abcxdefgh".to_owned()));
}

#[test]
fn text_area_end_on_a_wrapped_row_of_wide_characters_stays_on_that_row() {
    let mut state = text_area("界界界界");
    render(&mut state, FormLayout::Stacked, 4, 6);
    press(&mut state, &[KeyCode::End, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界x界界界".to_owned()));
}

#[test]
#[ignore]
fn text_area_cursor_at_the_end_of_a_full_row_stays_inside_the_area() {
    let mut state = text_area("abcd");
    render(&mut state, FormLayout::Stacked, 4, 6);
    state.handle_input(ctrl(KeyCode::End));
    assert!(screen_x(&mut state, 4) < 4);
}

#[test]
#[ignore]
fn text_area_cursor_at_the_end_of_a_full_row_of_wide_characters_stays_inside_the_area() {
    let mut state = text_area("界界");
    render(&mut state, FormLayout::Stacked, 4, 6);
    state.handle_input(ctrl(KeyCode::End));
    assert!(screen_x(&mut state, 4) < 4);
}

#[test]
fn text_area_up_from_ascii_into_wide_characters_keeps_the_visual_column() {
    let mut state = text_area("界界界\nabcdef");
    render(&mut state, FormLayout::Stacked, 20, 6);
    state.handle_input(ctrl(KeyCode::Home));
    press(&mut state, &[KeyCode::Down]);
    press(&mut state, &[KeyCode::Right; 4]);
    press(&mut state, &[KeyCode::Up, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界界x界\nabcdef".to_owned()));
}

#[test]
fn text_area_down_from_wide_characters_into_ascii_keeps_the_visual_column() {
    let mut state = text_area("界界\nabcdef");
    render(&mut state, FormLayout::Stacked, 20, 6);
    state.handle_input(ctrl(KeyCode::Home));
    press(&mut state, &[KeyCode::Right, KeyCode::Right]);
    press(&mut state, &[KeyCode::Down, KeyCode::Char('x')]);
    assert_eq!(state.value(&1), Some("界界\nabcdxef".to_owned()));
}

#[test]
fn text_area_vertical_move_never_lands_in_the_middle_of_a_wide_character() {
    let mut state = text_area("abc\n界界");
    render(&mut state, FormLayout::Stacked, 20, 6);
    state.handle_input(ctrl(KeyCode::Home));
    press(
        &mut state,
        &[KeyCode::Right, KeyCode::Down, KeyCode::Char('x')],
    );
    assert_eq!(state.value(&1), Some("abc\nx界界".to_owned()));
}

fn cursor_and_index(state: &FormState<i32>) -> ((u16, u16), usize) {
    let text_area = state.text_area(&1).unwrap();
    (text_area.cursor_position(), text_area.index_position())
}

#[test]
fn text_area_cursor_position_is_column_and_row_in_cells() {
    let mut state = text_area("abc\n界界");
    render(&mut state, FormLayout::Stacked, 20, 6);
    state.handle_input(ctrl(KeyCode::Home));
    press(&mut state, &[KeyCode::Down, KeyCode::Right]);
    assert_eq!(cursor_and_index(&state), ((2, 1), 5));
}

#[test]
fn text_area_cursor_position_follows_the_wrapping() {
    let mut state = text_area("界界界");
    render(&mut state, FormLayout::Stacked, 4, 6);
    state.handle_input(ctrl(KeyCode::End));
    assert_eq!(cursor_and_index(&state), ((2, 1), 3));
}

#[test]
fn text_area_cursor_position_ascii_is_unchanged() {
    let mut state = text_area("abc");
    render(&mut state, FormLayout::Stacked, 20, 6);
    state.handle_input(ctrl(KeyCode::End));
    assert_eq!(cursor_and_index(&state), ((3, 0), 3));
}
