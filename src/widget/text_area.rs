use std::borrow::Cow;

use ratatui::{
    buffer::{Buffer, CellWidth},
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    text::Line,
    widgets::{
        Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
    },
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions},
    field_builder_common,
    style::{FormStyle, Parts, States, Widgets},
};

// BUILDER
/// Builder for a multi-line text field, started with
/// [`FormBuilder::text_area`](crate::builder::FormBuilder::text_area).
/// Like the other field builders, it supports the common options
/// `required`, `optional`, `disabled`, `readonly`, `height`,
/// `validator`, and `normalizer`.
pub struct TextAreaBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) options: FieldOptions,
    pub(crate) placeholder: Option<String>,
    pub(crate) scrollbar: bool,
}

impl<T: PartialEq> TextAreaBuilder<T> {
    /// Sets the field's initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Sets a hint shown while the field is empty, styled with
    /// `FormStyle::placeholder`. Disappears as soon as the user types
    /// anything, and has no effect on validation
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Enables or disables the vertical scrollbar for this text area.
    pub fn scrollbar(mut self, show_scrollbar: bool) -> Self {
        self.scrollbar = show_scrollbar;
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        let initial_value = self.value.to_string();
        // Starts at the beginning, not the end (unlike SingleLine): with a
        // long, pre-filled, multi-line value, starting at the end could
        // land the cursor scrolled past content the user hasn't seen yet.
        let position = 0;
        self.form.push_field(Field {
            id: self.id,
            kind: FieldKind::TextArea(TextAreaStatus {
                label: self.label,
                value: self.value,
                index: position,
                lines: Vec::new(),
                placeholder: self.placeholder,
                visible_height: 0,
                scrollbar: self.scrollbar,
            }),
            options: self.options,
            error: None,
            initial_value,
        });

        self.form
    }
}
field_builder_common!(TextAreaBuilder<T>);

/// A read-only view into a text_area field's state.
#[derive(Debug, Copy, Clone)]
pub struct TextAreaRef<'a> {
    pub(crate) inner: &'a TextAreaStatus,
}

impl TextAreaRef<'_> {
    /// The field's current value, including newlines.
    pub fn value(&self) -> &str {
        &self.inner.value
    }

    /// The cursor's `(column, row)` in the wrapped text as last rendered,
    /// with the column in terminal cells. It is not a screen position (see
    /// [`FormState::cursor_position`]) and it is `(0, 0)` before the first
    /// render; [`index_position`](Self::index_position) doesn't depend on
    /// rendering.
    pub fn cursor_position(&self) -> (u16, u16) {
        calculate_coordinate(self.inner)
    }

    /// The cursor's position within the value, in characters from the start
    /// (each newline counts as one).
    pub fn index_position(&self) -> usize {
        self.inner.index
    }

    /// The content split into visual rows, as last rendered, without
    /// trailing newlines. Empty before the first render.
    pub fn lines(&self) -> Vec<&str> {
        self.inner.lines.iter().map(|(_, l)| l.as_str()).collect()
    }

    /// The number of visual rows in [`lines`](Self::lines).
    pub fn line_count(&self) -> usize {
        self.inner.lines.len()
    }

    /// How many lines the view has scrolled down to keep the cursor visible.
    ///
    /// `0` while the cursor's line is already within the visible area;
    /// otherwise, the number of lines above the visible area's top.
    ///
    /// Only meaningful after the field has been rendered at least once —
    /// before that, the field doesn't yet know how much vertical space it
    /// has available, and this returns `0` regardless of the cursor's
    /// actual position.
    pub fn scroll_offset(&self) -> u16 {
        let (_, row) = calculate_coordinate(self.inner);
        (row + 1).saturating_sub(self.inner.visible_height)
    }
}

// STATUS
#[derive(Debug)]
pub struct TextAreaStatus {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) index: usize,
    pub(crate) lines: Vec<(usize, String)>,
    pub(crate) placeholder: Option<String>,
    pub(crate) visible_height: u16,
    pub(crate) scrollbar: bool,
}

impl TextAreaStatus {
    pub(crate) fn get(&self) -> String {
        self.value.clone()
    }

    pub(crate) fn get_ref(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.value.as_ref())
    }

    pub(crate) fn set(&mut self, value: &str) {
        let old_position = self.index;
        self.value = value.to_owned();
        self.index = old_position.min(self.value.chars().count());
    }

    fn byte_position(&self, position: usize, default: usize) -> usize {
        self.value
            .char_indices()
            .nth(position)
            .map_or(default, |(i, _)| i)
    }
    fn delete(&mut self) {
        if self.value.is_empty() {
            return;
        }
        let byte_idx = self.byte_position(self.index, self.value.len());
        if byte_idx < self.value.len() {
            self.value.remove(byte_idx);
            self.index = self.index.min(self.value.chars().count());
        }
    }
    fn backspace(&mut self) {
        if self.index == 0 {
            return;
        }
        let byte_idx = self.byte_position(self.index - 1, 0);
        self.value.remove(byte_idx);
        self.index = self.index.saturating_sub(1)
    }
    fn left(&mut self) {
        self.index = self.index.saturating_sub(1)
    }
    fn right(&mut self) {
        self.index = (self.index + 1).min(self.value.chars().count());
    }
    fn home(&mut self) {
        self.index = 0
    }
    fn end(&mut self) {
        self.index = self.value.chars().count();
    }
    fn up(&mut self) {
        let (col, mut row) = calculate_coordinate(self);
        row = row.saturating_sub(1);
        self.index = calculate_position(self, col, row);
    }
    fn down(&mut self) {
        let (col, mut row) = calculate_coordinate(self);
        if row + 1 < self.lines.len() as u16 {
            row += 1;
        }
        self.index = calculate_position(self, col, row);
    }
    fn enter(&mut self) {
        self.insert('\n');
    }
    fn insert(&mut self, c: char) {
        let byte_idx = self.byte_position(self.index, self.value.len());
        self.value.insert(byte_idx, c);
        self.index += 1;
    }

    fn begin_row(&mut self) {
        let (_, row) = calculate_coordinate(self);
        self.index = calculate_position(self, 0, row);
    }

    fn end_row(&mut self) {
        let (_, row) = calculate_coordinate(self);
        let next = self
            .lines
            .get(row as usize + 1)
            .map(|(s, _)| s)
            .copied()
            .unwrap_or_default();

        self.index = self
            .lines
            .get(row as usize)
            .map(|(start, l)| {
                let len = l.chars().count();
                let size = start + len;
                if next == size {
                    start + len.saturating_sub(1)
                } else {
                    start + len
                }
            })
            .unwrap_or_default();
    }

    fn page_up(&mut self) {
        let (col, row) = calculate_coordinate(self);
        let new_row = row.saturating_sub(self.visible_height.saturating_sub(1));
        self.index = calculate_position(self, col, new_row);
    }

    fn page_down(&mut self) {
        let (col, row) = calculate_coordinate(self);
        let last_row = self.lines.len().saturating_sub(1) as u16;
        let new_row = (row + self.visible_height.saturating_sub(1)).min(last_row);
        self.index = calculate_position(self, col, new_row);
    }
}

// EVENT
pub(crate) fn handle_input_textarea(key_event: KeyEvent, text_area: &mut TextAreaStatus) {
    match (key_event.modifiers, key_event.code) {
        (_, KeyCode::Backspace) => text_area.backspace(),
        (_, KeyCode::Left) => text_area.left(),
        (_, KeyCode::Right) => text_area.right(),
        (KeyModifiers::CONTROL, KeyCode::Home) => text_area.home(),
        (_, KeyCode::Home) => text_area.begin_row(),
        (KeyModifiers::CONTROL, KeyCode::End) => text_area.end(),
        (_, KeyCode::End) => text_area.end_row(),
        (_, KeyCode::Delete) => text_area.delete(),
        (_, KeyCode::Up) => text_area.up(),
        (_, KeyCode::Down) => text_area.down(),
        (_, KeyCode::Char(c))
            if !key_event
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            text_area.insert(c)
        }
        (_, KeyCode::Enter) => text_area.enter(),
        (_, KeyCode::PageUp) => text_area.page_up(),
        (_, KeyCode::PageDown) => text_area.page_down(),
        _ => {}
    }
}

// RENDER
pub(crate) fn render_textarea(
    area: Rect,
    buf: &mut Buffer,
    text_area: &mut TextAreaStatus,
    style: &FormStyle,
    state: States,
) -> Option<(u16, u16)> {
    let mut text_style = style.get(Widgets::SINGLE_LINE, Parts::TEXT, state);

    let scrollbar_gap = if text_area.scrollbar { 1 } else { 0 };
    text_area.lines = wrap_text(
        &text_area.value,
        area.width.saturating_sub(scrollbar_gap) as usize,
    );

    let mut display = text_area
        .lines
        .iter()
        .map(|(_, line)| Line::from(line.clone()))
        .collect::<Vec<_>>();

    if let Some(placeholder) = text_area.placeholder.as_ref()
        && display.is_empty()
    {
        display = vec![Line::from(placeholder.clone())];
        text_style = style.get(Widgets::SINGLE_LINE, Parts::PLACEHOLDER, state);
    }

    let (col, row) = calculate_coordinate(text_area);

    let scroll_y = (row + 1).saturating_sub(area.height);

    let value = Paragraph::new(display)
        .style(text_style)
        .block(Block::default().style(style.get(Widgets::SINGLE_LINE, Parts::AREA, state)))
        .scroll((scroll_y, 0));

    value.render(area, buf);

    if text_area.scrollbar {
        let mut scroll_state = ScrollbarState::new(text_area.lines.len())
            .viewport_content_length(area.height as usize)
            .position(row as usize);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        scrollbar.render(area, buf, &mut scroll_state);
    }

    Some((area.x + col, area.y + row.saturating_sub(scroll_y)))
}

fn calculate_coordinate(text_area: &TextAreaStatus) -> (u16, u16) {
    let mut row: u16 = 0;
    let mut begin: usize = 0;
    let mut max_length: usize = 0;

    for (start, line) in &text_area.lines {
        if *start > text_area.index {
            break;
        }
        begin = *start;
        max_length = line.chars().count();
        row += 1;
    }
    let pos = (text_area.index.saturating_sub(begin)).min(max_length);
    let col = text_area
        .value
        .chars()
        .skip(begin)
        .take(pos)
        .collect::<String>()
        .cell_width();
    (col, row.saturating_sub(1))
}

fn calculate_position(text_area: &TextAreaStatus, col: u16, row: u16) -> usize {
    if let Some((start, line)) = text_area.lines.get(row as usize) {
        let mut cur = 0;
        let mut index = 0;
        let mut buf = [0; 4];
        for ch in line.chars() {
            let delta = ch.encode_utf8(&mut buf).cell_width();
            if cur + delta > col {
                break;
            }
            index += 1;
            cur += delta;
        }
        let x = index.min(line.chars().count());
        *start + x
    } else {
        0
    }
}

fn wrap_by_width(text: &str, width: u16) -> Vec<String> {
    let mut buf = [0; 4];
    let mut s = String::new();
    let mut result = Vec::new();
    for ch in text.chars() {
        let delta = ch.encode_utf8(&mut buf).cell_width();
        if !s.is_empty() && s.cell_width() + delta > width {
            result.push(std::mem::take(&mut s));
        }
        s.push(ch);
    }
    if !s.is_empty() {
        result.push(s);
    }
    result
}

fn wrap_text(text: &str, width: usize) -> Vec<(usize, String)> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut pos = 0;
    for line in text.lines() {
        if line.is_empty() {
            lines.push((pos, "".to_string()));
            pos += 1;
            continue;
        }
        for block in wrap_by_width(line, width as u16) {
            if !block.is_empty() {
                let new_pos = pos + block.chars().count();
                lines.push((pos, block));
                pos = new_pos;
            }
        }
        pos += 1;
    }

    if let Some(c) = text.chars().last()
        && c == '\n'
    {
        lines.push((pos, "".to_string()));
    }

    lines
}

#[cfg(test)]
mod tests_wrap_text {
    use super::*;

    #[test]
    fn line_shorter_than_width_single_chunk() {
        let result = wrap_text("hello world", 20);
        assert_eq!(result, vec![(0, "hello world".to_string())]);
    }

    #[test]
    fn length_is_exact_multiple_of_width() {
        let result = wrap_text("abcdefghij", 5);
        assert_eq!(
            result,
            vec![(0, "abcde".to_string()), (5, "fghij".to_string())]
        );
    }

    #[test]
    fn length_with_remainder_shorter_last_chunk() {
        let result = wrap_text("abcdefghijk", 5);
        assert_eq!(
            result,
            vec![
                (0, "abcde".to_string()),
                (5, "fghij".to_string()),
                (10, "k".to_string()),
            ]
        );
    }

    #[test]
    fn splits_regardless_of_word_boundaries() {
        let result = wrap_text("the quick", 4);
        assert_eq!(
            result,
            vec![
                (0, "the ".to_string()),
                (4, "quic".to_string()),
                (8, "k".to_string()),
            ]
        );
    }

    #[test]
    fn width_one_each_chunk_is_a_single_char() {
        let result = wrap_text("abc", 1);
        assert_eq!(
            result,
            vec![
                (0, "a".to_string()),
                (1, "b".to_string()),
                (2, "c".to_string()),
            ]
        );
    }

    #[test]
    fn multiple_lines_global_positions() {
        let result = wrap_text("hello\nworld12345", 5);
        assert_eq!(
            result,
            vec![
                (0, "hello".to_string()),
                (6, "world".to_string()),
                (11, "12345".to_string()),
            ]
        );
    }

    #[test]
    fn empty_line_in_the_middle() {
        let result = wrap_text("ab\n\ncd", 5);
        assert_eq!(
            result,
            vec![
                (0, "ab".to_string()),
                (3, "".to_string()),
                (4, "cd".to_string()),
            ]
        );
    }

    #[test]
    fn empty_string_returns_empty_vec() {
        let result = wrap_text("", 5);
        assert_eq!(result, Vec::<(usize, String)>::new());
    }

    // #[test]
    // fn trailing_newline_produces_empty_last_line() {
    //     let result = wrap_text("abc\n", 5);
    //     assert_eq!(result, vec![(0, "abc".to_string()), (4, "".to_string())]);
    // }

    #[test]
    fn whitespace_only_line_is_not_a_special_case() {
        let result = wrap_text("   ", 5);
        assert_eq!(result, vec![(0, "   ".to_string())]);
    }

    #[test]
    fn width_and_positions_count_unicode_chars_not_bytes() {
        let result = wrap_text("àèìòùé", 3);
        assert_eq!(result, vec![(0, "àèì".to_string()), (3, "òùé".to_string())]);
    }

    #[test]
    fn multiple_trailing_empty_lines() {
        let result = wrap_text("abc\n\n\n", 5);
        assert_eq!(
            result,
            vec![
                (0, "abc".to_string()),
                (4, "".to_string()),
                (5, "".to_string()),
                (6, "".to_string()),
            ]
        );
    }

    #[test]
    fn leading_empty_line() {
        let result = wrap_text("\nabc", 5);
        assert_eq!(result, vec![(0, "".to_string()), (1, "abc".to_string())]);
    }

    #[test]
    fn text_made_only_of_newlines() {
        let result = wrap_text("\n\n", 5);
        assert_eq!(
            result,
            vec![
                (0, "".to_string()),
                (1, "".to_string()),
                (2, "".to_string()),
            ]
        );
    }
}

#[cfg(test)]
mod coordinate_tests {
    use super::*;

    fn make_text_area(lines: &[(usize, &str)], position: u16) -> TextAreaStatus {
        // Rebuild the value the rows were wrapped from: a row that starts
        // right where the previous one ended is a soft wrap, a gap is a '\n'.
        let mut value = String::new();
        let mut chars = 0;
        for (start, line) in lines {
            while chars < *start {
                value.push('\n');
                chars += 1;
            }
            value.push_str(line);
            chars += line.chars().count();
        }
        TextAreaStatus {
            label: "Test".to_owned(),
            value,
            index: position as usize,
            lines: lines
                .iter()
                .map(|(start, line)| (*start, (*line).to_owned()))
                .collect(),
            placeholder: None,
            visible_height: 5,
            scrollbar: false,
        }
    }

    #[test]
    fn coordinate_in_the_middle_of_the_second_line() {
        // "hello" (0..5) wrapped into "world" (5..10): position 7 is the
        // third character of the second line.
        let text_area = make_text_area(&[(0, "hello"), (5, "world")], 7);
        assert_eq!(calculate_coordinate(&text_area), (2, 1));
    }

    #[test]
    fn coordinate_exactly_on_a_line_boundary_belongs_to_the_next_line() {
        // Position 5 is "5 characters to the left of the cursor" -- i.e.
        // right after a 5-character first line, which is the START of the
        // second line, not the end of the first.
        let text_area = make_text_area(&[(0, "hello"), (5, "world")], 5);
        assert_eq!(calculate_coordinate(&text_area), (0, 1));
    }

    #[test]
    fn coordinate_does_not_panic_when_lines_is_empty() {
        // `lines` starts empty in `finish()` and is only populated by the
        // first render -- if `Up`/`Down` is ever handled before the first
        // render, this must not panic (it does today: `y - 1` underflows
        // on `u16` when the loop never runs).
        let text_area = make_text_area(&[], 0);
        assert_eq!(calculate_coordinate(&text_area), (0, 0));
    }

    #[test]
    fn position_clamps_x_to_the_length_of_the_target_line() {
        // Asking for column 10 on a 5-character line should clamp to the
        // end of that line, not walk off the end of the string.
        let text_area = make_text_area(&[(0, "hello"), (5, "hi")], 0);
        assert_eq!(calculate_position(&text_area, 10, 1), 7);
    }

    #[test]
    fn position_returns_zero_for_a_row_beyond_the_last_line() {
        let text_area = make_text_area(&[(0, "hello")], 0);
        assert_eq!(calculate_position(&text_area, 0, 5), 0);
    }

    #[test]
    fn coordinate_and_position_round_trip_for_an_in_bounds_value() {
        let text_area = make_text_area(&[(0, "hello"), (5, "world")], 8);
        let (x, y) = calculate_coordinate(&text_area);
        assert_eq!(calculate_position(&text_area, x, y), 8);
    }
}

#[cfg(test)]
mod editing_tests {
    use ratatui::crossterm::event::KeyModifiers;

    use super::*;

    fn make_text_area(value: &str, position: u16) -> TextAreaStatus {
        TextAreaStatus {
            label: "Test".to_owned(),
            value: value.to_owned(),
            index: position as usize,
            lines: Vec::new(),
            placeholder: None,
            visible_height: 0,
            scrollbar: false,
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn insert_handles_unicode_content_correctly() {
        // "città" = c-i-t-t-à, position 3 sits right before the second 't'.
        let mut text_area = make_text_area("città", 3);
        handle_input_textarea(key(KeyCode::Char('X')), &mut text_area);

        assert_eq!(text_area.value, "citXtà");
        assert_eq!(text_area.index, 4);
    }

    #[test]
    fn backspace_removes_the_character_before_the_cursor() {
        // Cursor at the end: backspace must remove the 'à' (multi-byte)
        // as a whole character, not split it or remove the wrong byte.
        let mut text_area = make_text_area("città", 5);
        handle_input_textarea(key(KeyCode::Backspace), &mut text_area);

        assert_eq!(text_area.value, "citt");
        assert_eq!(text_area.index, 4);
    }

    #[test]
    fn backspace_at_the_start_does_not_panic_or_change_anything() {
        let mut text_area = make_text_area("città", 0);
        handle_input_textarea(key(KeyCode::Backspace), &mut text_area);

        assert_eq!(text_area.value, "città");
        assert_eq!(text_area.index, 0);
    }

    #[test]
    fn delete_removes_the_character_at_the_cursor() {
        // Position 4 sits right before the 'à'. Delete must remove it.
        let mut text_area = make_text_area("città", 4);
        handle_input_textarea(key(KeyCode::Delete), &mut text_area);

        assert_eq!(text_area.value, "citt");
        assert_eq!(text_area.index, 4);
    }

    #[test]
    fn delete_at_the_end_of_the_value_does_not_remove_anything() {
        let mut text_area = make_text_area("citt", 4);
        handle_input_textarea(key(KeyCode::Delete), &mut text_area);

        assert_eq!(text_area.value, "citt");
    }

    #[test]
    fn delete_on_an_empty_value_does_not_panic() {
        let mut text_area = make_text_area("", 0);
        handle_input_textarea(key(KeyCode::Delete), &mut text_area);

        assert_eq!(text_area.value, "");
        assert_eq!(text_area.index, 0);
    }

    #[test]
    fn left_does_not_go_below_zero() {
        let mut text_area = make_text_area("ciao", 0);
        handle_input_textarea(key(KeyCode::Left), &mut text_area);

        assert_eq!(text_area.index, 0);
    }

    #[test]
    fn right_does_not_go_past_the_end() {
        let mut text_area = make_text_area("ciao", 4);
        handle_input_textarea(key(KeyCode::Right), &mut text_area);

        assert_eq!(text_area.index, 4);
    }

    #[test]
    fn backspace_at_the_start_of_a_line_merges_it_with_the_previous_line() {
        // "abc\ndef" -- position 4 is right after the newline, at the
        // start of "def". Backspace must remove the '\n' itself, joining
        // the two lines into "abcdef", not just refuse to do anything
        // because there's no "character" to the left on this visual row.
        let mut text_area = make_text_area("abc\ndef", 4);
        handle_input_textarea(key(KeyCode::Backspace), &mut text_area);

        assert_eq!(text_area.value, "abcdef");
        assert_eq!(text_area.index, 3);
    }

    #[test]
    fn delete_at_the_end_of_a_line_merges_it_with_the_next_line() {
        // Same idea, the other direction: position 3 is right before the
        // newline, at the end of "abc". Delete must remove the '\n'.
        let mut text_area = make_text_area("abc\ndef", 3);
        handle_input_textarea(key(KeyCode::Delete), &mut text_area);

        assert_eq!(text_area.value, "abcdef");
        assert_eq!(text_area.index, 3);
    }

    #[test]
    fn ctrl_and_alt_do_not_insert_the_character() {
        let mut text_area = make_text_area("", 0);

        handle_input_textarea(
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            &mut text_area,
        );
        handle_input_textarea(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT),
            &mut text_area,
        );

        assert_eq!(text_area.value, ""); // niente inserito
    }

    #[test]
    fn shift_still_inserts_the_character() {
        let mut text_area = make_text_area("", 0);

        handle_input_textarea(
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT),
            &mut text_area,
        );

        assert_eq!(text_area.value, "A"); // Shift non blocca, coerente con gli altri due
    }
}
