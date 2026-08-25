use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::Style,
    widgets::{List, ListState, StatefulWidget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    field::{Field, FieldKind, FieldOptions},
    field_builder_common,
};

// BUILDER
/// Builder for a select field: a list of options the user picks from with
/// the arrow keys. Started with
/// [`FormBuilder::select`](crate::builder::FormBuilder::select). For the
/// options shared with every other field kind, see
/// [`field_builder_common`](crate::field_builder_common).
pub struct SelectBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: usize,
    pub(crate) options: FieldOptions,
}

impl<T: PartialEq> SelectBuilder<T> {
    /// Sets which option is selected initially, by index into the list of
    /// values.
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the list of `(value, label)` pairs from a slice of borrowed
    /// strings — the ergonomic choice for a literal list, e.g.
    /// `&[("I", "Italia"), ("F", "Francia")]`. `value` is what
    /// `value()`/`values()` return once selected; `label` is what's shown
    /// on screen. See [`SelectBuilder::values`] for owned or dynamically
    /// built data.
    pub fn values_ref(mut self, input: &[(&str, &str)]) -> Self {
        self.values = input
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();

        self
    }

    /// Sets the list of `(value, label)` pairs from any iterator of
    /// owned-or-convertible pairs — a `Vec<(String, String)>`, a
    /// `HashMap`, or anything else `IntoIterator`. Prefer
    /// [`SelectBuilder::values_ref`] for a literal list of borrowed
    /// strings.
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

    fn finish(mut self) -> FormBuilder<T> {
        let initial_value = self
            .values
            .get(self.selected)
            .map(|(k, _)| k.clone())
            .unwrap_or_default();
        self.form.fields.push(Field {
            id: self.id,
            kind: FieldKind::Select(SelectStatus {
                label: self.label,
                values: self.values,
                list_state: ListState::default().with_selected(Some(self.selected)),
            }),
            options: self.options,
            error: None,
            initial_value,
        });

        self.form
    }
}
field_builder_common!(SelectBuilder<T>);

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

    pub(crate) fn set(&mut self, value: &str) {
        let index = self.values.iter().position(|(k, _)| k == value);
        self.list_state.select(index);
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
    value_style: Style,
    highlight_style: Style,
) -> Option<(u16, u16)> {
    let items: Vec<_> = select.values.iter().map(|(_, v)| v.as_str()).collect();

    let list = List::new(items)
        .style(value_style)
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    StatefulWidget::render(list, area, buf, &mut select.list_state);

    None
}

#[cfg(test)]
mod select_tests {
    use super::*;

    fn make_select(values: &[(&str, &str)], selected: Option<usize>) -> SelectStatus {
        SelectStatus {
            label: "Test".to_owned(),
            values: values
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            list_state: match selected {
                Some(idx) => ListState::default().with_selected(Some(idx)),
                None => ListState::default(),
            },
        }
    }

    #[test]
    fn set_then_get_round_trips_the_selected_value() {
        let mut select = make_select(
            &[("I", "Italia"), ("F", "Francia"), ("D", "Germania")],
            Some(0),
        );
        select.set("F");
        assert_eq!(select.get(), "F");
    }

    #[test]
    fn set_with_no_matching_value_deselects_everything() {
        // A value that isn't in the list doesn't leave the current
        // selection untouched -- it clears it entirely. The same
        // "reset, not no-op" surprise already found in Checkbox::set().
        let mut select = make_select(&[("I", "Italia"), ("F", "Francia")], Some(0));
        select.set("nonexistent");
        assert_eq!(select.get(), "");
    }

    #[test]
    fn get_returns_empty_string_when_nothing_is_selected() {
        // Through the builder, list_state always starts with a selection,
        // so in practice this only happens if nothing was ever selected --
        // but it's exactly the path that makes `required()` meaningful at
        // all for a Select field.
        let select = make_select(&[("I", "Italia")], None);
        assert_eq!(select.get(), "");
    }
}
