use std::clone;

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{List, ListState, StatefulWidget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions, Requirement},
    field_builder_common,
};

// BUILDER
pub struct SelectBuilder {
    pub(crate) form: FormBuilder,
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: usize,
    pub(crate) options: FieldOptions,
}

impl SelectBuilder {
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    pub fn values_ref(mut self, input: &[(&str, &str)]) -> Self {
        self.values = input
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();

        self
    }

    pub fn values<I, K, V>(mut self, input: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.values = input
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();

        self
    }

    fn finish(mut self) -> FormBuilder {
        self.form.fields.push(Field {
            kind: FieldKind::Select(SelectStatus {
                label: self.label,
                values: self.values,
                list_state: ListState::default().with_selected(Some(self.selected)),
            }),
            options: self.options,
            error: None,
        });

        self.form
    }
}
field_builder_common!(SelectBuilder);

// STATUS
pub struct SelectStatus {
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) list_state: ListState,
}

impl SelectStatus {
    pub(crate) fn get(&self) -> String {
        self.list_state
            .selected()
            .and_then(|idx| self.values.get(idx).map(|(k, _)| k.clone()))
            .unwrap_or_default()
    }

    fn up(&mut self) {
        self.list_state.select_previous();
    }

    fn down(&mut self) {
        self.list_state.select_next();
    }

    fn home(&mut self) {
        self.list_state.select_first();
    }

    fn end(&mut self) {
        self.list_state.select_last();
    }

    fn page_up(&mut self) {
        self.list_state.scroll_up_by(8);
    }

    fn page_down(&mut self) {
        self.list_state.scroll_down_by(8);
    }
}

// EVENT
pub(crate) fn handle_input_select(key_code: KeyCode, select: &mut SelectStatus) {
    match key_code {
        KeyCode::Up => select.up(),
        KeyCode::Down => select.down(),
        KeyCode::Home => select.home(),
        KeyCode::End => select.end(),
        KeyCode::PageUp => select.page_up(),
        KeyCode::PageDown => select.page_down(),
        _ => {}
    }
}

// RENDER
pub(crate) fn render_select(
    area: Rect,
    buf: &mut Buffer,
    select: &mut SelectStatus,
    has_focus: bool,
) -> Option<(u16, u16)> {
    let items: Vec<_> = select.values.iter().map(|(k, v)| v.as_str()).collect();

    let style = Style::default().fg(Color::Gray);

    let list = List::new(items)
        .style(style)
        .highlight_style(if has_focus {
            Modifier::REVERSED
        } else {
            Modifier::ITALIC
        })
        .highlight_symbol("> ");

    StatefulWidget::render(list, area, buf, &mut select.list_state);

    None
}
