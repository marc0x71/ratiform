use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::Style,
    widgets::{List, ListState, StatefulWidget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    error::BuildError,
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
    pub(crate) selected: Option<usize>,
    pub(crate) options: FieldOptions,
}

impl<T: PartialEq> SelectBuilder<T> {
    /// Sets which option is selected initially, by index into the list of
    /// values.
    ///
    /// `selected` isn't validated against the list length at this point —
    /// if it's a valid index once [`build`](crate::builder::FormBuilder::build)
    /// runs, it's used as given; if it's out of range for however many
    /// values the field ends up with, it's silently clamped to the last
    /// one instead of panicking or leaving the field unselected.
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = Some(selected);
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

    /// Starts the field with nothing selected, instead of the first
    /// option (the default). This is the only way `required()` has any
    /// effect on a `Select` — with a selection always present otherwise,
    /// `required()` can never fail.
    ///
    /// Mutually exclusive with [`selected`](SelectBuilder::selected):
    /// whichever is called last wins.
    pub fn no_selection(mut self) -> Self {
        self.selected = None;
        self
    }

    fn validate_field(&mut self) {
        if self.form.pending_error.is_some() {
            return;
        }
        for i in 0..self.values.len() {
            for j in (i + 1)..self.values.len() {
                if self.values[i].0 == self.values[j].0 {
                    self.form.pending_error = Some(BuildError::DuplicateSelectValue {
                        first: i,
                        duplicate: j,
                    });
                    return;
                }
            }
        }
    }

    fn finish(mut self) -> FormBuilder<T> {
        self.validate_field();
        let initial_value = self
            .selected
            .and_then(|sel| self.values.get(sel).map(|(k, _)| k.clone()))
            .unwrap_or_default();
        self.form.push_field(Field {
            id: self.id,
            kind: FieldKind::Select(SelectStatus {
                label: self.label,
                values: self.values,
                list_state: ListState::default().with_selected(self.selected),
                height: self.options.height,
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
#[derive(Debug)]
pub struct SelectStatus {
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) list_state: ListState,
    pub(crate) height: u16,
}

impl SelectStatus {
    pub(crate) fn get(&self) -> String {
        let last = self.values.len().saturating_sub(1);
        self.list_state
            .selected()
            .and_then(|idx| self.values.get(idx.min(last)))
            .map(|(k, _)| k.clone())
            .unwrap_or_default()
    }

    pub(crate) fn get_ref(&self) -> Cow<'_, str> {
        let last = self.values.len().saturating_sub(1);
        self.list_state
            .selected()
            .and_then(|idx| self.values.get(idx.min(last)))
            .map(|(k, _)| Cow::Borrowed(k.as_ref()))
            .unwrap_or(Cow::Borrowed(""))
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
        self.list_state.scroll_up_by(self.height.saturating_sub(1));
    }

    fn page_down(&mut self) {
        self.list_state
            .scroll_down_by(self.height.saturating_sub(1));
    }
}

// EVENT
pub(crate) fn handle_input_select(key_event: KeyEvent, select: &mut SelectStatus) {
    match key_event.code {
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
            height: 5,
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

#[cfg(test)]
mod builder_select_tests {
    use crate::{builder::FormBuilder, error::BuildError};

    #[test]
    fn selected_within_bounds_behaves_as_before() {
        let state = FormBuilder::new()
            .select(1, "Paese")
            .values_ref(&[("I", "Italia"), ("F", "Francia")])
            .selected(1)
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some("F".to_owned()));
        assert_eq!(state.is_field_dirty(&1), Some(false));
    }

    #[test]
    fn selected_out_of_range_clamps_instead_of_producing_a_dirty_field() {
        let state = FormBuilder::new()
            .select(1, "Paese")
            .values_ref(&[("I", "Italia"), ("F", "Francia"), ("D", "Germania")])
            .selected(99) // fuori range: solo indici 0..=2 esistono
            .build()
            .unwrap();

        // Deve corrispondere all'ultima opzione (coerente col clamp di get()),
        // non alla stringa vuota.
        assert_eq!(state.value(&1), Some("D".to_owned()));
        // E soprattutto: non deve nascere già "sporco".
        assert_eq!(state.is_field_dirty(&1), Some(false));
    }

    #[test]
    fn selected_on_an_empty_list_stays_empty_and_not_dirty() {
        let state = FormBuilder::new()
            .select(1, "Paese")
            .selected(5) // nessuna opzione esiste comunque
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some(String::new()));
        assert_eq!(state.is_field_dirty(&1), Some(false));
    }
    #[test]
    fn no_selection_starts_the_field_empty() {
        let state = FormBuilder::new()
            .select(1, "Paese")
            .values_ref(&[("I", "Italia"), ("F", "Francia")])
            .no_selection()
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some(String::new()));
    }

    #[test]
    fn without_no_selection_the_default_first_option_is_unchanged() {
        // Non-regressione: il comportamento di sempre, per chi non chiama
        // no_selection(), deve restare identico.
        let state = FormBuilder::new()
            .select(1, "Paese")
            .values_ref(&[("I", "Italia"), ("F", "Francia")])
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some("I".to_owned()));
        assert_eq!(state.is_field_dirty(&1), Some(false));
    }

    #[test]
    fn explicit_selected_still_works_after_the_internal_type_change() {
        let state = FormBuilder::new()
            .select(1, "Paese")
            .values_ref(&[("I", "Italia"), ("F", "Francia")])
            .selected(1)
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some("F".to_owned()));
    }

    #[test]
    fn calling_both_selected_and_no_selection_the_last_one_wins() {
        let state = FormBuilder::new()
            .select(1, "Paese")
            .values_ref(&[("I", "Italia"), ("F", "Francia")])
            .selected(1)
            .no_selection() // chiamato per ultimo -> vince questo
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some(String::new()));
    }

    #[test]
    fn duplicate_select_values_are_caught_even_when_select_is_not_the_last_field() {
        let result = FormBuilder::new()
            .select(1, "Paese")
            .values_ref(&[("I", "Italia"), ("I", "Italia bis")]) // valore duplicato: "I"
            .checkbox(2, "Accetto i termini") // <- un campo DOPO il Select
            .build();

        assert!(result.is_err()); // oggi, con questa patch, è Ok(...)
    }

    #[test]
    fn duplicate_select_reports_the_first_pair_not_the_last() {
        let result = FormBuilder::new()
            .select(1, "Paese")
            .values_ref(&[("I", "a"), ("I", "b"), ("I", "c")]) // "I" ripetuto 3 volte
            .build();

        assert_eq!(
            result.err().unwrap(),
            BuildError::DuplicateSelectValue {
                first: 0,
                duplicate: 1
            }
        );
    }
}
