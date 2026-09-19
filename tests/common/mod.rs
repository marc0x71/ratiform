#![allow(dead_code)]

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    widgets::StatefulWidget,
};
use ratiform::{Form, FormLayout, FormState};

pub fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Renders `state` into a fresh `w` x `h` buffer, the way a frame would.
pub fn render(state: &mut FormState<i32>, layout: FormLayout<i32>, w: u16, h: u16) -> Buffer {
    let area = Rect::new(0, 0, w, h);
    let mut buf = Buffer::empty(area);
    Form::default()
        .with_layout(layout)
        .render(area, &mut buf, state);
    buf
}

/// The symbol drawn in a single cell.
pub fn cell(buf: &Buffer, x: u16, y: u16) -> String {
    buf[(x, y)].symbol().to_owned()
}

/// The text of a whole row, with surrounding spaces removed.
pub fn row(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect::<String>()
        .trim()
        .to_owned()
}

pub fn words(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_owned).collect()
}
