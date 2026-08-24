#![allow(unused)]
pub mod builder;
mod event;
mod field;
mod render;
pub mod style;
pub mod validators;
mod widget;

use std::{hash::Hash, marker::PhantomData};

use field::Field;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    style::Stylize,
    text::Span,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crate::{event::handle_input_field, style::FormStyle};

#[derive(Default, Clone, Copy)]
pub enum FormResult {
    Submitted,
    Cancelled,
    #[default]
    Working,
}

#[derive(Default)]
pub struct FormState<T> {
    fields: Vec<Field<T>>,
    focus: usize,
    cursor_position: Option<(u16, u16)>,
    result: FormResult,
}

impl<T: PartialEq> FormState<T> {
    pub(crate) fn new(mut fields: Vec<Field<T>>) -> Self {
        fields.iter_mut().for_each(|f| f.validate());
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
            .max_by_key(|c| c.label().chars().count())
            .map(|f| f.label().chars().count())
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
            KeyCode::Enter if !self.has_errors() => self.result = FormResult::Submitted,
            KeyCode::Esc => self.result = FormResult::Cancelled,
            KeyCode::Tab if !self.fields.is_empty() => {
                self.focus = self.focus.wrapping_add(1) % self.fields.len();
            }
            KeyCode::BackTab if !self.fields.is_empty() => {
                self.focus = self.focus.wrapping_sub(1) % self.fields.len();
            }
            _ => {
                if !self.fields.is_empty()
                    && let Some(field) = self.fields.get_mut(self.focus)
                    && !field.options.disabled
                    && !field.options.readonly
                {
                    handle_input_field(key_event.code, field);
                }
            }
        }
    }

    pub fn result(&self) -> FormResult {
        self.result
    }

    fn has_errors(&self) -> bool {
        self.fields.iter().any(|f| f.has_error())
    }

    pub fn values(self) -> impl Iterator<Item = (T, String)> {
        self.fields.into_iter().map(|f| {
            let value = f.get();
            (f.id, value)
        })
    }

    pub fn value(&self, id: &T) -> Option<String> {
        self.fields.iter().find(|&f| f.id == *id).map(|f| f.get())
    }

    pub fn set_value(&mut self, id: &T, value: &str) {
        if let Some(f) = self.fields.iter_mut().find(|f| f.id == *id) {
            f.set(value);
        }
    }

    pub fn focus_field(&self) -> Option<&T> {
        self.fields.get(self.focus).map(|f| &f.id)
    }
}

pub struct Form<T> {
    style: FormStyle,
    _phantom: PhantomData<T>,
}

impl<T> Form<T> {
    pub fn with_style(style: FormStyle) -> Self {
        Self {
            style,
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for Form<T> {
    fn default() -> Self {
        Self {
            style: FormStyle::default(),
            _phantom: PhantomData,
        }
    }
}
