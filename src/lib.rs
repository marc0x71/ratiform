#![allow(unused)]
pub mod builder;
mod field;
mod render;

use std::hash::Hash;

use field::Field;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    style::Stylize,
    text::Span,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

#[derive(Default, Clone, Copy)]
pub enum FormResult {
    Submitted,
    Cancelled,
    #[default]
    Working,
}

#[derive(Default)]
pub struct FormState {
    fields: Vec<Field>,
    focus: usize,
    cursor_position: Option<(u16, u16)>,
    result: FormResult,
}

impl FormState {
    pub(crate) fn new(fields: Vec<Field>) -> Self {
        Self {
            fields,
            focus: 0,
            cursor_position: None,
            result: FormResult::Working,
        }
    }

    pub(crate) fn max_label_length(&self) -> usize {
        self.fields
            .iter()
            .max_by_key(|c| c.label().len())
            .map(|f| f.label().len())
            .unwrap_or_default()
    }

    pub fn cursor_position(&self) -> Option<(u16, u16)> {
        self.cursor_position
    }

    pub(crate) fn set_cursor_position(&mut self, cursor_position: Option<(u16, u16)>) {
        self.cursor_position = cursor_position;
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }
        match key_event.code {
            KeyCode::Enter => self.result = FormResult::Submitted,
            KeyCode::Esc => self.result = FormResult::Cancelled,
            KeyCode::Tab if !self.fields.is_empty() => {
                self.focus = self.focus.wrapping_add(1) % self.fields.len();
            }
            KeyCode::BackTab if !self.fields.is_empty() => {
                self.focus = self.focus.wrapping_sub(1) % self.fields.len();
            }
            _ => {}
        }
    }

    pub fn result(&self) -> FormResult {
        self.result
    }
}

#[derive(Default)]
pub struct Form {}
