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
    internal::list::{HorizontalList, HorizontalListState},
};

pub(crate) enum SelectDirection {
    Horizontal,
    Vertical,
}

// BUILDER
/// Builder for a select field: a list of options the user picks from with
/// the arrow keys. Started with
/// [`FormBuilder::select`](crate::builder::FormBuilder::select).
/// Like the other field builders, it supports the common options
/// `required`, `optional`, `disabled`, `readonly`, `height`,
/// `validator`, and `normalizer`.
pub struct SelectBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: Option<usize>,
    pub(crate) options: FieldOptions,
    pub(crate) highlight_symbol: String,
    pub(crate) direction: SelectDirection,
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

    /// Sets the symbol shown before the currently selected row.
    ///
    /// Applied only to the highlighted list item; it does not affect the
    /// symbols or spacing of unselected rows. Has no effect when the field
    /// has no selection (see [`no_selection`](Self::no_selection)), since in
    /// that state no row is highlighted.
    ///
    /// Defaults to `"> "`.
    pub fn highlight_symbol(mut self, highlight_symbol: impl Into<String>) -> Self {
        self.highlight_symbol = highlight_symbol.into();
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

    pub fn horizontal(mut self) -> Self {
        self.direction = SelectDirection::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.direction = SelectDirection::Vertical;
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        self.validate_field();
        let initial_value = self
            .selected
            .and_then(|sel| self.values.get(sel).map(|(k, _)| k.clone()))
            .unwrap_or_default();
        let list_state = match self.direction {
            SelectDirection::Horizontal => SelectStateDirection::Horizontal(
                HorizontalListState::default().with_selected(self.selected),
            ),
            SelectDirection::Vertical => {
                SelectStateDirection::Vertical(ListState::default().with_selected(self.selected))
            }
        };
        self.form.push_field(Field {
            id: self.id,
            kind: FieldKind::Select(SelectStatus {
                label: self.label,
                values: self.values,
                list_state,
                height: self.options.height,
                highlight_symbol: self.highlight_symbol,
            }),
            options: self.options,
            error: None,
            initial_value,
        });

        self.form
    }
}
field_builder_common!(SelectBuilder<T>);

/// A read-only view into a select field's state.
#[derive(Debug, Copy, Clone)]
pub struct SelectRef<'a> {
    pub(crate) inner: &'a SelectStatus,
}

impl SelectRef<'_> {
    /// The index of the currently selected option, or `None` if there is no
    /// selection (e.g. the field has no options).
    pub fn selected_index(&self) -> Option<usize> {
        self.inner.list_state.selected()
    }

    /// The *value* of the currently selected option — the first element of
    /// the `(value, label)` pair — or `None` if there is no selection.
    ///
    /// This is what [`FormState::value`] returns for a select field. Use
    /// [`selected_label`](SelectRef::selected_label) instead for the text
    /// shown on screen.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ratiform::builder::FormBuilder;
    /// # #[derive(Debug, PartialEq, Eq, Hash)]
    /// # enum Field { Country }
    /// let state = FormBuilder::new()
    ///     .select(Field::Country, "Country")
    ///     .values_ref(&[("IT", "Italy"), ("FR", "France")])
    ///     .selected(1)
    ///     .build()
    ///     .unwrap();
    ///
    /// let sel = state.select(&Field::Country).unwrap();
    /// assert_eq!(sel.selected_value(), Some("FR"));
    /// assert_eq!(sel.selected_label(), Some("France"));
    /// ```
    pub fn selected_label(&self) -> Option<&str> {
        self.inner
            .list_state
            .selected()
            .and_then(|idx| self.inner.values.get(idx))
            .map(|(_, s)| s.as_str())
    }

    /// The *label* of the currently selected option — the second element of
    /// the `(value, label)` pair, i.e. the text shown on screen — or `None`
    /// if there is no selection.
    ///
    /// See [`selected_value`](SelectRef::selected_value) for an example
    /// contrasting the two.
    pub fn selected_value(&self) -> Option<&str> {
        self.inner
            .list_state
            .selected()
            .and_then(|idx| self.inner.values.get(idx))
            .map(|(s, _)| s.as_str())
    }
}

// STATUS
#[derive(Debug)]
pub(crate) enum SelectStateDirection {
    Horizontal(HorizontalListState),
    Vertical(ListState),
}
impl SelectStateDirection {
    fn selected(&self) -> Option<usize> {
        match self {
            SelectStateDirection::Horizontal(state) => state.selected(),
            SelectStateDirection::Vertical(state) => state.selected(),
        }
    }

    fn select(&mut self, index: Option<usize>) {
        match self {
            SelectStateDirection::Horizontal(state) => state.select(index),
            SelectStateDirection::Vertical(state) => state.select(index),
        }
    }

    fn select_previous(&mut self) {
        match self {
            SelectStateDirection::Horizontal(state) => state.select_previous(),
            SelectStateDirection::Vertical(state) => state.select_previous(),
        }
    }

    fn select_next(&mut self) {
        match self {
            SelectStateDirection::Horizontal(state) => state.select_next(),
            SelectStateDirection::Vertical(state) => state.select_next(),
        }
    }

    fn select_first(&mut self) {
        match self {
            SelectStateDirection::Horizontal(state) => state.select_first(),
            SelectStateDirection::Vertical(state) => state.select_first(),
        }
    }

    fn select_last(&mut self) {
        match self {
            SelectStateDirection::Horizontal(state) => state.select_last(),
            SelectStateDirection::Vertical(state) => state.select_last(),
        }
    }

    fn scroll_up_by(&mut self, amount: u16) {
        match self {
            SelectStateDirection::Horizontal(state) => state.scroll_up_by(amount),
            SelectStateDirection::Vertical(state) => state.scroll_up_by(amount),
        }
    }

    fn scroll_down_by(&mut self, amount: u16) {
        match self {
            SelectStateDirection::Horizontal(state) => state.scroll_down_by(amount),
            SelectStateDirection::Vertical(state) => state.scroll_down_by(amount),
        }
    }
}

#[derive(Debug)]
pub struct SelectStatus {
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) list_state: SelectStateDirection,
    pub(crate) height: u16,
    pub(crate) highlight_symbol: String,
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

    fn right(&mut self) {
        if matches!(self.list_state, SelectStateDirection::Horizontal(_)) {
            self.list_state.select_next();
        }
    }

    fn left(&mut self) {
        if matches!(self.list_state, SelectStateDirection::Horizontal(_)) {
            self.list_state.select_previous();
        }
    }

    fn up(&mut self) {
        if matches!(self.list_state, SelectStateDirection::Vertical(_)) {
            self.list_state.select_previous();
        }
    }

    fn down(&mut self) {
        if matches!(self.list_state, SelectStateDirection::Vertical(_)) {
            self.list_state.select_next();
        }
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
        KeyCode::Left => select.left(),
        KeyCode::Right => select.right(),
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

    match select.list_state {
        SelectStateDirection::Horizontal(ref mut horizontal_list_state) => {
            let list = HorizontalList::new(items)
                .style(value_style)
                .highlight_style(highlight_style);

            StatefulWidget::render(list, area, buf, horizontal_list_state);
        }
        SelectStateDirection::Vertical(ref mut list_state) => {
            let list = List::new(items)
                .style(value_style)
                .highlight_style(highlight_style)
                .highlight_symbol(select.highlight_symbol.as_str());
            StatefulWidget::render(list, area, buf, list_state);
        }
    }

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
                Some(idx) => {
                    SelectStateDirection::Vertical(ListState::default().with_selected(Some(idx)))
                }
                None => SelectStateDirection::Vertical(ListState::default()),
            },
            height: 5,
            highlight_symbol: "> ".to_string(),
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

    #[test]
    fn select_ref_distinguishes_value_from_label() {
        #[derive(Debug, PartialEq, Eq, Hash)]
        enum Field {
            Country,
            Notes,
        }
        let state = FormBuilder::new()
            .select(Field::Country, "Country")
            .values_ref(&[("IT", "Italy"), ("FR", "France")])
            .selected(1)
            .text_area(Field::Notes, "Notes")
            .build()
            .unwrap();

        let sel = state.select(&Field::Country).unwrap();

        assert_eq!(sel.selected_index(), Some(1));
        assert_eq!(sel.selected_value(), Some("FR")); // primo elemento della coppia
        assert_eq!(sel.selected_label(), Some("France")); // secondo elemento, quello mostrato a schermo
    }

    #[test]
    fn select_ref_reports_none_when_no_options() {
        #[derive(Debug, PartialEq, Eq, Hash)]
        enum Field {
            Country,
            Notes,
        }
        let state = FormBuilder::new()
            .select(Field::Country, "Country")
            .values_ref(&[])
            .no_selection()
            .text_area(Field::Notes, "Notes")
            .build()
            .unwrap();

        let sel = state.select(&Field::Country).unwrap();

        assert_eq!(sel.selected_index(), None);
        assert_eq!(sel.selected_value(), None);
        assert_eq!(sel.selected_label(), None);
    }
}
