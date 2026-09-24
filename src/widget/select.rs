use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    error::BuildError,
    field::{Field, FieldKind, FieldOptions},
    field_builder_common,
    internal::list::HorizontalList,
    style::{FormStyle, Parts, States, Widgets},
    widget::common::{
        direction::{Direction, StateDirection},
        searchable::Search,
    },
};

// BUILDER
/// Builder for a select field: a list of options the user picks from.
/// Started with
/// [`FormBuilder::select`](crate::builder::FormBuilder::select).
/// Like the other field builders, it supports the common options
/// `required`, `optional`, `disabled`, `readonly`, `height`,
/// `validator`, and `normalizer`.
///
/// Vertical by default — navigated with `Up`/`Down`/`Home`/`End`/
/// `PageUp`/`PageDown`. [`horizontal`](Self::horizontal) switches to a
/// single scrolling row navigated with `Left`/`Right` (plus `Home`/`End`)
/// instead.
pub struct SelectBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: Option<usize>,
    pub(crate) options: FieldOptions,
    pub(crate) highlight_symbol: String,
    pub(crate) direction: Direction,
    pub(crate) spacing: usize,
    pub(crate) preview: usize,
    pub(crate) scrollbar: bool,
    pub(crate) searchable: bool,
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

    /// Sets the symbol shown before the option under the cursor. Ignored when
    /// the field is [`horizontal`](Self::horizontal) — there, the cursor is
    /// shown only via `highlight_style`.
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

    /// Lays the options out on a single scrolling row instead of a column,
    /// navigated with `Left`/`Right`, and `Home`/`End` jump to the first and
    /// last option. `Up`/`Down` and `PageUp`/`PageDown` have no effect.
    pub fn horizontal(mut self) -> Self {
        self.direction = Direction::Horizontal;
        self
    }

    /// Lays the options out in a column — the default. Reverses
    /// [`horizontal`](Self::horizontal).
    pub fn vertical(mut self) -> Self {
        self.direction = Direction::Vertical;
        self
    }

    /// Enables or disables scrollbar for this select.
    pub fn scrollbar(mut self, show_scrollbar: bool) -> Self {
        self.scrollbar = show_scrollbar;
        self
    }

    /// Turns the option list into a filter-as-you-type combobox: typed
    /// characters narrow the options down by a fuzzy, case-insensitive
    /// match on the label, and `Esc` clears the query before it cancels
    /// the form.
    pub fn searchable(mut self) -> Self {
        self.searchable = true;
        self
    }

    /// Number of options to keep visible past the selected one when
    /// scrolling horizontally. Ignored when the field is
    /// [`vertical`](Self::vertical).
    ///
    /// Defaults to `2`.
    pub fn preview(mut self, preview: usize) -> Self {
        self.preview = preview;
        self
    }

    /// Number of spaces between options. Ignored when the field is
    /// [`vertical`](Self::vertical).
    ///
    /// Defaults to `2`.
    pub fn spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }

    fn finish(mut self) -> FormBuilder<T> {
        self.validate_field();
        let initial_value = self
            .selected
            .and_then(|sel| self.values.get(sel).map(|(k, _)| k.clone()))
            .unwrap_or_default();
        let list_state = StateDirection::new(self.direction, self.selected);
        if self.scrollbar && matches!(self.direction, Direction::Horizontal) {
            self.options.height = self.options.height.max(2)
        }
        let len = self.values.len();
        self.form.push_field(Field {
            id: self.id,
            kind: FieldKind::Select(SelectStatus {
                label: self.label,
                values: self.values,
                list_state,
                height: self.options.height,
                highlight_symbol: self.highlight_symbol,
                spacing: self.spacing,
                preview: self.preview,
                scrollbar: if self.scrollbar {
                    Some(ScrollbarState::new(len))
                } else {
                    None
                },
                searchable: if self.searchable {
                    Search::enabled(len)
                } else {
                    Search::Disabled
                },
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
        self.inner.selected()
    }

    /// The *label* of the currently selected option — the second element of
    /// the `(value, label)` pair, i.e. the text shown on screen — or `None`
    /// if there is no selection.
    ///
    /// See [`selected_value`](SelectRef::selected_value) for an example
    /// contrasting the two.
    pub fn selected_label(&self) -> Option<&str> {
        self.inner
            .selected()
            .and_then(|idx| self.inner.values.get(idx))
            .map(|(_, s)| s.as_str())
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
    pub fn selected_value(&self) -> Option<&str> {
        self.inner
            .selected()
            .and_then(|idx| self.inner.values.get(idx))
            .map(|(s, _)| s.as_str())
    }

    /// The user's current search query, if `.searchable()` is enabled —
    /// `Some("")` before any character is typed, `None` if the field
    /// isn't searchable at all.
    pub fn search_query(&self) -> Option<&str> {
        self.inner.searchable.search_query()
    }

    /// The number of options matching the current search query.
    ///
    /// Returns the total number of options when the field isn't searchable or
    /// when the search query is empty.
    pub fn filtered_count(&self) -> usize {
        self.inner
            .searchable
            .filtered_count(self.inner.values.len())
    }
}

#[derive(Debug)]
pub struct SelectStatus {
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) list_state: StateDirection,
    pub(crate) height: u16,
    pub(crate) highlight_symbol: String,
    pub(crate) spacing: usize,
    pub(crate) preview: usize,
    pub(crate) scrollbar: Option<ScrollbarState>,
    pub(crate) searchable: Search,
}

impl SelectStatus {
    fn selected(&self) -> Option<usize> {
        self.list_state.selected().and_then(|idx| {
            self.searchable
                .original_index_at(idx, self.values.iter().len())
        })
    }

    pub(crate) fn get(&self) -> String {
        self.selected()
            .and_then(|idx| self.values.get(idx))
            .map(|(k, _)| k.clone())
            .unwrap_or_default()
    }

    pub(crate) fn get_ref(&self) -> Cow<'_, str> {
        self.selected()
            .and_then(|idx| self.values.get(idx))
            .map(|(k, _)| Cow::Borrowed(k.as_ref()))
            .unwrap_or(Cow::Borrowed(""))
    }

    pub(crate) fn set(&mut self, value: &str) {
        self.searchable.reset();
        let original = self.values.iter().position(|(k, _)| k == value);
        let position = self.searchable.filtered_index_of(original);
        self.list_state.select(position);
    }

    fn refilter(&mut self) {
        let values_iter = self.values.iter().map(|(_, v)| v.as_str());
        self.searchable.refilter(values_iter);
    }

    pub(crate) fn special_key_handled(&self) -> Vec<KeyCode> {
        if self.searchable.has_query() {
            vec![KeyCode::Esc]
        } else {
            Vec::new()
        }
    }
}

// EVENT
pub(crate) fn handle_input_select(key_event: KeyEvent, select: &mut SelectStatus) {
    let need_refilter = select.searchable.handle_input(key_event);
    if need_refilter {
        select.refilter();
    } else {
        select.list_state.handle_input(key_event, select.height);
    }
}

fn make_spans<'a>(
    text: &'a str,
    positions: &[usize],
    normal: Style,
    highlight: Style,
) -> Vec<Span<'a>> {
    text.char_indices()
        .enumerate()
        .map(|(char_pos, (byte_pos, ch))| {
            let end = byte_pos + ch.len_utf8();
            let slice = &text[byte_pos..end];

            if positions.contains(&char_pos) {
                Span::styled(slice, highlight)
            } else {
                Span::styled(slice, normal)
            }
        })
        .collect()
}

// RENDER
pub(crate) fn render_select(
    area: Rect,
    buf: &mut Buffer,
    select: &mut SelectStatus,
    style: &FormStyle,
    field_state: States,
) -> Option<(u16, u16)> {
    let list_area =
        if select.scrollbar.is_some() && matches!(select.list_state, StateDirection::Vertical(_)) {
            Rect {
                width: area.width.saturating_sub(1),
                ..area
            }
        } else {
            area
        };

    let normal = style.get(Widgets::SELECT, Parts::ITEM, field_state);
    let highlight = style.get(Widgets::SELECT, Parts::MATCH, field_state);
    let items: Vec<Line<'_>> = select
        .searchable
        .iter(select.values.len())
        .map(|(idx, positions)| {
            let text = select.values[idx].1.as_str();
            Line::from(make_spans(text, positions, normal, highlight))
        })
        .collect();

    match select.list_state {
        StateDirection::Horizontal(ref mut list_state) => {
            let list = HorizontalList::new(items, select.spacing, select.preview)
                .highlight_style(style.get(Widgets::SELECT, Parts::ACTIVE, field_state));

            StatefulWidget::render(list, list_area, buf, list_state);
        }
        StateDirection::Vertical(ref mut list_state) => {
            let list = List::new(items)
                .highlight_style(style.get(Widgets::SELECT, Parts::ACTIVE, field_state))
                .highlight_symbol(select.highlight_symbol.as_str());
            StatefulWidget::render(list, list_area, buf, list_state);
        }
    }

    if let Some(ref mut scroll_state) = select.scrollbar {
        let (orientation, selected, [begin_sym, end_sym]) = match &select.list_state {
            StateDirection::Horizontal(list_state) => (
                ScrollbarOrientation::HorizontalBottom,
                list_state.selected().unwrap_or_default(),
                ["←", "→"],
            ),
            StateDirection::Vertical(list_state) => (
                ScrollbarOrientation::VerticalRight,
                list_state.selected().unwrap_or_default(),
                ["↑", "↓"],
            ),
        };
        *scroll_state = ScrollbarState::new(select.values.len()).position(selected);

        let scrollbar = Scrollbar::new(orientation)
            .style(style.get(Widgets::SELECT, Parts::ITEM, field_state))
            .begin_symbol(Some(begin_sym))
            .end_symbol(Some(end_sym));

        scrollbar.render(area, buf, scroll_state);
    }

    None
}

#[cfg(test)]
mod select_tests {
    use ratatui::crossterm::event::KeyModifiers;

    use super::*;

    fn make_select(values: &[(&str, &str)], selected: Option<usize>) -> SelectStatus {
        SelectStatus {
            label: "Test".to_owned(),
            values: values
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            list_state: StateDirection::new(Direction::Vertical, selected),
            height: 5,
            highlight_symbol: "> ".to_string(),
            spacing: 2,
            preview: 2,
            scrollbar: None,
            searchable: Search::Disabled,
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

    #[test]
    fn selected_label_clamps_after_select_last_before_any_render() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );

        handle_input_select(KeyEvent::new(KeyCode::End, KeyModifiers::NONE), &mut select);

        let select_ref = SelectRef { inner: &select };
        assert_eq!(select_ref.selected_label(), Some("Germania"));
        assert_eq!(select_ref.selected_value(), Some("DE"));
    }

    #[test]
    fn selected_never_exceeds_last_valid_index_even_with_bogus_list_state() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );
        select.list_state.select(Some(usize::MAX));
        assert_eq!(select.selected(), Some(2)); // ultimo indice valido, 3 opzioni
    }

    #[test]
    fn selected_is_none_without_a_list_state_selection() {
        let select = make_select(&[("IT", "Italia"), ("FR", "Francia")], None);
        assert_eq!(select.selected(), None);
    }

    #[test]
    fn selected_clamps_stale_cursor_after_refilter() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );

        select.searchable = Search::enabled(select.values.len());
        select.list_state.select(Some(2)); // cursore su "Germania"

        for c in "fra".chars() {
            select
                .searchable
                .handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }

        select.refilter();

        assert_eq!(select.selected(), Some(1)); // indice originale di Francia
    }

    #[test]
    fn page_down_moves_by_the_field_height_minus_one() {
        let mut select = make_select(&[("a", "A"), ("b", "B")], Some(0)); // height: 5
        handle_input_select(
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            &mut select,
        );
        assert_eq!(select.list_state.selected(), Some(4));
    }

    #[test]
    fn typing_in_search_does_not_move_cursor() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );
        select.searchable = Search::enabled(select.values.len());
        select.list_state.select(Some(0));

        handle_input_select(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            &mut select,
        );

        assert_eq!(select.searchable.search_query(), Some("a"));
        assert_eq!(select.list_state.selected(), Some(0));
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
