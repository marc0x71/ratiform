#![allow(unused)]

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::Style,
    text::{self, Line},
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions},
    field_builder_common,
};

// BUILDER
pub struct TextAreaBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) options: FieldOptions,
}

impl<T: PartialEq> TextAreaBuilder<T> {
    /// Sets the field's initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        let initial_value = self.value.to_string();
        let position = self.value.len() as u16;
        self.form.fields.push(Field {
            id: self.id,
            kind: FieldKind::TextArea(TextAreaStatus {
                label: self.label,
                value: self.value,
                position,
                lines: Vec::new(),
            }),
            options: self.options,
            error: None,
            initial_value,
        });

        self.form
    }
}
field_builder_common!(TextAreaBuilder<T>);

// STATUS
pub struct TextAreaStatus {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) position: u16,
    pub(crate) lines: Vec<(usize, String)>,
}

impl TextAreaStatus {
    pub(crate) fn get(&self) -> String {
        self.value.clone()
    }

    pub(crate) fn set(&mut self, value: &str) {
        self.value = value.to_owned()
    }

    fn byte_position(&self, position: u16, default: usize) -> usize {
        self.value
            .char_indices()
            .nth(position as usize)
            .map_or(default, |(i, _)| i)
    }
    fn delete(&mut self) {
        if self.value.is_empty() {
            return;
        }
        let byte_idx = self.byte_position(self.position, self.value.len());
        if byte_idx < self.value.len() {
            self.value.remove(byte_idx);
            self.position = self.position.min(self.value.chars().count() as u16)
        }
    }
    fn backspace(&mut self) {
        if self.position == 0 {
            return;
        }
        let byte_idx = self.byte_position(self.position - 1, 0);
        self.value.remove(byte_idx);
        self.position = self.position.saturating_sub(1)
    }
    fn left(&mut self) {
        self.position = self.position.saturating_sub(1)
    }
    fn right(&mut self) {
        self.position = (self.position + 1).min(self.value.chars().count() as u16)
    }
    fn home(&mut self) {
        self.position = 0
    }
    fn end(&mut self) {
        self.position = self.value.chars().count() as u16
    }
    fn up(&mut self) {
        let (x, mut y) = calculate_coordinate(self);
        y = y.saturating_sub(1);
        self.position = calculate_position(self, x, y);
    }
    fn down(&mut self) {
        let (x, mut y) = calculate_coordinate(self);
        if y + 1 < self.lines.len() as u16 {
            y += 1;
        }
        self.position = calculate_position(self, x, y);
    }
    fn enter(&mut self) {
        self.insert('\n');
    }
    fn insert(&mut self, c: char) {
        let byte_idx = self.byte_position(self.position, self.value.len());
        self.value.insert(byte_idx, c);
        self.position += 1;
    }
}

// EVENT
pub(crate) fn handle_input_textarea(key_code: KeyCode, text_area: &mut TextAreaStatus) {
    match key_code {
        KeyCode::Backspace => text_area.backspace(),
        KeyCode::Left => text_area.left(),
        KeyCode::Right => text_area.right(),
        KeyCode::Home => text_area.home(),
        KeyCode::End => text_area.end(),
        KeyCode::Delete => text_area.delete(),
        KeyCode::Up => text_area.up(),
        KeyCode::Down => text_area.down(),
        KeyCode::Char(c) => text_area.insert(c),
        KeyCode::Enter => text_area.enter(),
        _ => {}
    }
}

// RENDER
pub(crate) fn render_textarea(
    area: Rect,
    buf: &mut Buffer,
    text_area: &mut TextAreaStatus,
    value_style: Style,
    highlight_style: Style,
    _placeholder_style: Style,
) -> Option<(u16, u16)> {
    let style = value_style;

    text_area.lines = wrap_text(&text_area.value, area.width as usize);
    let v: Vec<_> = text_area.lines.iter().map(|(x, _)| *x).collect();

    let display = text_area
        .lines
        .iter()
        .map(|(_, line)| Line::from(line.clone()))
        .collect::<Vec<_>>();

    // TODO: da fare placeholder

    let value = Paragraph::new(display)
        .style(style)
        .block(Block::default().style(highlight_style));

    value.render(area, buf);

    let (x, y) = calculate_coordinate(text_area);
    Some((area.x + x, area.y + y))
}

fn calculate_coordinate(text_area: &TextAreaStatus) -> (u16, u16) {
    let mut y: u16 = 0;
    let mut begin: u16 = 0;

    for (start, _) in &text_area.lines {
        if *start as u16 > text_area.position {
            break;
        }
        begin = *start as u16;
        y += 1;
    }
    let x = text_area.position - begin;
    (x, y - 1)
}

fn calculate_position(text_area: &TextAreaStatus, x: u16, y: u16) -> u16 {
    if let Some((start, line)) = text_area.lines.get(y as usize) {
        let x = x.min(line.chars().count() as u16);
        (*start as u16) + x
    } else {
        0
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<(usize, String)> {
    let mut lines = Vec::new();
    let mut pos = 0;
    for line in text.lines() {
        if line.is_empty() {
            lines.push((pos, "".to_string()));
            pos += 1;
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        for block in chars.chunks(width) {
            let block: String = block.iter().collect();
            if !block.is_empty() {
                let new_pos = pos + block.chars().count();
                lines.push((pos, block));
                pos = new_pos;
            }
        }
        pos += 1;
    }

    lines
}

// fn wrap_text__(text: &str, width: usize) -> Vec<(usize, String)> {
//     let mut lines = Vec::new();
//     let mut pos = 0;
//     dbg!(&text, width);
//
//     for row in text.lines() {
//         let mut line = String::new();
//         for word in row.split_whitespace() {
//             let word_len = word.chars().count();
//
//             // too long word
//             if word_len >= width {
//                 if !line.is_empty() {
//                     let new_pos = pos + line.chars().count() + 1;
//                     lines.push((pos, std::mem::take(&mut line)));
//                     pos = new_pos;
//                 }
//
//                 let chars: Vec<char> = word.chars().collect();
//                 for block in chars.chunks(width) {
//                     if block.len() < width {
//                         line = block.iter().collect();
//                         break;
//                     }
//                     let block: String = block.iter().collect();
//                     if !block.is_empty() {
//                         // ma serve?!?!!?
//                         let new_pos = pos + block.chars().count();
//                         lines.push((pos, block));
//                         pos = new_pos;
//                     }
//                 }
//                 continue;
//             }
//
//             // check length
//             let delta = if line.is_empty() { 0 } else { 1 };
//             if line.chars().count() + word_len + delta > width {
//                 let new_pos = pos + line.chars().count() + delta;
//                 lines.push((pos, line));
//                 pos = new_pos;
//                 line = word.to_owned();
//                 continue;
//             }
//
//             // append
//             if !line.is_empty() {
//                 line.push(' ');
//             }
//             line.push_str(word);
//         }
//
//         if !line.is_empty() {
//             let new_pos = pos + line.chars().count();
//             lines.push((pos, line));
//             pos = new_pos;
//         }
//         pos += 1;
//     }
//     dbg!(&lines);
//     lines
// }

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
}
