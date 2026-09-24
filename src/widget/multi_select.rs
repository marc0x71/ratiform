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
/// Builder for a multi-select field: any number of options can be checked
/// with `Space`. Started with
/// [`FormBuilder::multi_select`](crate::builder::FormBuilder::multi_select).
/// Like the other field builders, it supports the common options
/// `required`, `optional`, `disabled`, `readonly`, `height`, `validator`,
/// and `normalizer`.
///
/// Cursor and selection are independent, unlike `Select` — moving the
/// cursor never changes what's selected.
///
/// Vertical by default — the cursor moves with `Up`/`Down`/`Home`/`End`/
/// `PageUp`/`PageDown`. [`horizontal`](Self::horizontal) switches to a
/// single scrolling row where the cursor moves with `Left`/`Right`
/// (plus `Home`/`End`) instead.
pub struct MultiSelectBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: Vec<usize>,
    pub(crate) options: FieldOptions,
    pub(crate) selected_symbol: String,
    pub(crate) unselected_symbol: String,
    pub(crate) direction: Direction,
    pub(crate) spacing: usize,
    pub(crate) preview: usize,
    pub(crate) scrollbar: bool,
    pub(crate) pinnable: bool,
    pub(crate) searchable: bool,
}

impl<T: PartialEq> MultiSelectBuilder<T> {
    /// Marks these indices as selected initially. Repeated calls
    /// accumulate rather than replace. An out-of-range index is silently
    /// ignored, unlike `Select`, which clamps.
    pub fn selected(mut self, selected: &[usize]) -> Self {
        self.selected.extend_from_slice(selected);
        self
    }

    /// Sets the list of `(value, label)` pairs from borrowed strings. See
    /// [`MultiSelectBuilder::values`] for owned data. No value may
    /// contain a comma or repeat another — either fails
    /// [`build`](crate::builder::FormBuilder::build).
    pub fn values_ref(mut self, input: &[(&str, &str)]) -> Self {
        self.values = input
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();

        self
    }

    /// Same as [`values_ref`](Self::values_ref), from any iterator of
    /// owned-or-convertible pairs.
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

    /// Starts the field with nothing selected. Mutually exclusive with
    /// [`selected`](Self::selected): whichever is called last wins.
    pub fn no_selection(mut self) -> Self {
        self.selected = Vec::new();
        self
    }

    /// Sets the symbols shown before a selected and an unselected option.
    /// Not width-checked — keep them the same length or options won't
    /// line up.
    ///
    /// Defaults to `"✓ "` and `"  "`.
    pub fn symbols(
        mut self,
        selected_symbol: impl Into<String>,
        unselected_symbol: impl Into<String>,
    ) -> Self {
        self.selected_symbol = selected_symbol.into();
        self.unselected_symbol = unselected_symbol.into();
        self
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

    /// Enables or disables scrollbar for this multi-select.
    pub fn scrollbar(mut self, show_scrollbar: bool) -> Self {
        self.scrollbar = show_scrollbar;
        self
    }

    /// Keeps every checked option pinned at the top of the list, in front
    /// of the unchecked ones, so a long list never hides what's already
    /// selected.
    pub fn pinnable(mut self) -> Self {
        self.pinnable = true;
        self
    }

    /// Same as [`SelectBuilder::searchable`](crate::widget::select::SelectBuilder::searchable),
    /// but checked options stay visible while typing only if `.pinnable()`
    /// is also set.
    pub fn searchable(mut self) -> Self {
        self.searchable = true;
        self
    }

    fn validate_field(&mut self) {
        if self.form.pending_error.is_some() {
            return;
        }
        for i in 0..self.values.len() {
            if self.values[i].0.contains(',') {
                self.form.pending_error = Some(BuildError::InvalidMultiSelectValue { position: i });
                return;
            }
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
        let mut selected = vec![false; self.values.len()];
        self.selected.iter().for_each(|pos| {
            if let Some(sel) = selected.get_mut(*pos) {
                *sel = true;
            }
        });

        let initial_value = self
            .values
            .iter()
            .zip(&selected)
            .filter_map(|(value, active)| {
                if *active {
                    Some(value.0.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        let list_state = StateDirection::new(self.direction, Some(0));

        if self.scrollbar && matches!(self.direction, Direction::Horizontal) {
            self.options.height = self.options.height.max(2)
        }
        let len = self.values.len();

        self.form.push_field(Field {
            id: self.id,
            kind: FieldKind::MultiSelect(MultiSelectStatus {
                label: self.label,
                values: self.values,
                list_state,
                height: self.options.height,
                selected_symbol: self.selected_symbol,
                unselected_symbol: self.unselected_symbol,
                selected,
                spacing: self.spacing,
                preview: self.preview,
                scrollbar: self.scrollbar,
                pinnable: self.pinnable,
                order: (0..len).collect(),
                searchable: if self.searchable {
                    Search::enabled(len)
                } else {
                    Search::Disabled
                },
                view: (0..len).collect(),
            }),
            options: self.options,
            error: None,
            initial_value,
        });

        self.form
    }
}
field_builder_common!(MultiSelectBuilder<T>);

/// A read-only view into a multi-select field's state.
#[derive(Debug, Copy, Clone)]
pub struct MultiSelectRef<'a> {
    pub(crate) inner: &'a MultiSelectStatus,
}

impl MultiSelectRef<'_> {
    /// The index the keyboard cursor is on, or `None` if there are no
    /// options. Not the selection — see [`selected`](Self::selected).
    pub fn selected_index(&self) -> Option<usize> {
        self.inner
            .list_state
            .selected()
            .and_then(|idx| self.inner.view.get(idx).copied())
    }

    /// Indices of every selected option, in list order — not selection
    /// order.
    pub fn selected(&self) -> impl Iterator<Item = usize> {
        self.inner
            .selected
            .iter()
            .enumerate()
            .filter_map(|(idx, active)| if *active { Some(idx) } else { None })
    }

    /// Labels of every selected option, in list order. See
    /// [`selected_values`](Self::selected_values) for what
    /// [`FormState::value`] returns.
    pub fn selected_labels(&self) -> impl Iterator<Item = &str> {
        self.inner
            .values
            .iter()
            .zip(&self.inner.selected)
            .filter_map(|(value, active)| {
                if *active {
                    Some(value.1.as_str())
                } else {
                    None
                }
            })
    }

    /// Values of every selected option, in list order — joining these
    /// with `,` reproduces [`FormState::value`] for this field.
    pub fn selected_values(&self) -> impl Iterator<Item = &str> {
        self.inner
            .values
            .iter()
            .zip(&self.inner.selected)
            .filter_map(|(value, active)| {
                if *active {
                    Some(value.0.as_str())
                } else {
                    None
                }
            })
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
    /// when the search query is empty. Pinned selected options that remain
    /// visible without matching the query are not included.
    pub fn filtered_count(&self) -> usize {
        self.inner
            .searchable
            .filtered_count(self.inner.values.len())
    }
}

#[derive(Debug)]
pub struct MultiSelectStatus {
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: Vec<bool>,
    pub(crate) list_state: StateDirection,
    pub(crate) height: u16,
    pub(crate) selected_symbol: String,
    pub(crate) unselected_symbol: String,
    pub(crate) spacing: usize,
    pub(crate) preview: usize,
    pub(crate) scrollbar: bool,
    pub(crate) pinnable: bool,
    pub(crate) order: Vec<usize>,
    pub(crate) searchable: Search,
    pub(crate) view: Vec<usize>,
}

impl MultiSelectStatus {
    pub(crate) fn get(&self) -> String {
        self.values
            .iter()
            .zip(&self.selected)
            .filter_map(|(value, active)| {
                if *active {
                    Some(value.0.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(crate) fn get_ref(&self) -> Cow<'_, str> {
        Cow::Owned(
            self.values
                .iter()
                .zip(&self.selected)
                .filter_map(|(value, active)| {
                    if *active {
                        Some(value.0.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(","),
        )
    }

    pub(crate) fn set(&mut self, value: &str) {
        self.selected = vec![false; self.values.len()];
        for s in value.split(',') {
            if let Some(index) = self.values.iter().position(|(k, _)| k == s) {
                self.selected[index] = true;
            }
        }
        self.reorder();
        if self.searchable.reset() {
            self.refilter();
        } else {
            self.rebuild_view();
        }
    }

    fn toggle(&mut self) {
        if let Some(order_pos) = self.list_state.selected()
            && let Some(&pos) = self.view.get(order_pos)
            && let Some(sel) = self.selected.get_mut(pos)
        {
            *sel = !*sel;
            self.reorder();
        }
    }

    fn reorder(&mut self) {
        if !self.pinnable {
            return;
        }
        let mut selected = vec![];
        let mut unselected = vec![];
        for (idx, sel) in self.selected.iter().enumerate() {
            if *sel {
                selected.push(idx);
            } else {
                unselected.push(idx);
            }
        }
        self.order = selected;
        self.order.extend(unselected);
    }

    pub(crate) fn special_key_handled(&self) -> Vec<KeyCode> {
        if self.searchable.has_query() {
            vec![KeyCode::Esc]
        } else {
            Vec::new()
        }
    }

    fn refilter(&mut self) {
        let values_iter = self.values.iter().map(|(_, v)| v.as_str());
        self.searchable.refilter(values_iter);
        self.rebuild_view();
    }

    fn rebuild_view(&mut self) {
        self.view = self
            .order
            .iter()
            .copied()
            .filter(|idx| {
                // if selected and pinnable, it remains visible
                // otherwise, it is visible only if it passes the filter
                (self.selected[*idx] && self.pinnable)
                    || self.searchable.contains_original_index(*idx)
            })
            .collect();
    }
}

// EVENT
pub(crate) fn handle_input_multiselect(key_event: KeyEvent, select: &mut MultiSelectStatus) {
    let mut need_refilter = false;
    match key_event.code {
        KeyCode::Char(' ') => {
            select.toggle();
            if select.pinnable {
                select.rebuild_view();
            }
        }
        KeyCode::Backspace if select.searchable.has_query() => {
            need_refilter = select.searchable.handle_input(key_event);
        }
        KeyCode::Esc if select.searchable.has_query() => {
            need_refilter = select.searchable.handle_input(key_event);
        }
        KeyCode::Char(_) if select.searchable.has_filter() => {
            need_refilter = select.searchable.handle_input(key_event);
        }
        _ => select.list_state.handle_input(key_event, select.height),
    }
    if need_refilter {
        select.refilter();
    }
}

// RENDER
pub(crate) fn render_multiselect(
    area: Rect,
    buf: &mut Buffer,
    select: &mut MultiSelectStatus,
    style: &FormStyle,
    field_state: States,
) -> Option<(u16, u16)> {
    let list_area = if select.scrollbar && matches!(select.list_state, StateDirection::Vertical(_))
    {
        Rect {
            width: area.width.saturating_sub(1),
            ..area
        }
    } else {
        area
    };

    let mut items: Vec<Line<'_>> = Vec::new();

    let normal = style.get(Widgets::MULTI_SELECT, Parts::ITEM, field_state);
    let selected_style =
        normal.patch(style.get(Widgets::MULTI_SELECT, Parts::SELECTED, field_state));
    let highlight = style.get(Widgets::MULTI_SELECT, Parts::MATCH, field_state);

    for idx in select.view.iter().copied() {
        let v = select.values[idx].1.as_str();
        let positions = select.searchable.find_positions(idx);
        let prefix = if select.selected[idx] {
            select.selected_symbol.as_str()
        } else {
            select.unselected_symbol.as_str()
        };

        let mut item = Line::from(vec![Span::styled(
            prefix,
            style.get(Widgets::MULTI_SELECT, Parts::MARKER, field_state),
        )]);
        let base = if select.selected[idx] {
            selected_style
        } else {
            normal
        };
        let label = make_spans(v, positions, base, highlight);
        item.extend(label);

        items.push(item);
    }

    match select.list_state {
        StateDirection::Horizontal(ref mut list_state) => {
            let list = HorizontalList::new(items, select.spacing, select.preview)
                .style(style.get(Widgets::MULTI_SELECT, Parts::ITEM, field_state))
                .highlight_style(style.get(Widgets::MULTI_SELECT, Parts::ACTIVE, field_state));

            StatefulWidget::render(list, list_area, buf, list_state);
        }
        StateDirection::Vertical(ref mut list_state) => {
            let list = List::new(items)
                .style(style.get(Widgets::MULTI_SELECT, Parts::ITEM, field_state))
                .highlight_style(style.get(Widgets::MULTI_SELECT, Parts::ACTIVE, field_state));
            StatefulWidget::render(list, list_area, buf, list_state);
        }
    }

    if select.scrollbar {
        // The selection must be read after the list render, which clamps an
        // out-of-range index (e.g. `usize::MAX` from `select_last()`).
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
        let mut scroll_state = ScrollbarState::new(select.view.len()).position(selected);

        let scrollbar = Scrollbar::new(orientation)
            .style(style.get(Widgets::MULTI_SELECT, Parts::ITEM, field_state))
            .begin_symbol(Some(begin_sym))
            .end_symbol(Some(end_sym));

        scrollbar.render(area, buf, &mut scroll_state);
    }

    None
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
#[cfg(test)]
mod test_helpers {

    use super::*;

    pub(crate) fn make_select(
        values: &[(&str, &str)],
        selected: Option<usize>,
        pinnable: bool,
    ) -> MultiSelectStatus {
        MultiSelectStatus {
            label: "Test".to_owned(),
            values: values
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            list_state: StateDirection::new(Direction::Vertical, selected),
            height: 5,
            selected_symbol: "> ".to_string(),
            unselected_symbol: "  ".to_string(),
            selected: vec![false; values.len()],
            spacing: 2,
            preview: 2,
            scrollbar: false,
            pinnable,
            order: (0..values.len()).collect(),
            searchable: Search::Disabled,
            view: (0..values.len()).collect(),
        }
    }
}

#[cfg(test)]
mod multiselect_toggle_test {

    use crate::widget::multi_select::test_helpers::make_select;

    #[test]
    fn toggle_dopo_up_senza_mai_renderizzare_non_va_in_panic() {
        let mut select = make_select(&[("a", "A"), ("b", "B")], None, false);

        select.list_state.up();
        select.toggle();

        assert_eq!(select.get(), ""); // e non deve aver selezionato nulla
    }
}

#[cfg(test)]
mod multiselect_test {
    use ratatui::crossterm::event::KeyModifiers;

    use super::test_helpers::make_select;
    use super::*;

    #[test]
    fn set_then_get_round_trips_in_values_order_not_input_order() {
        // "F,I" in ingresso, ma "I" viene prima di "F" nella lista -- get()
        // deve seguire l'ordine di `values`, non l'ordine passato a set().
        let mut select = make_select(
            &[("I", "Italia"), ("F", "Francia"), ("D", "Germania")],
            None,
            false,
        );
        select.set("F,I");
        assert_eq!(select.get(), "I,F");
    }

    #[test]
    fn set_with_one_unknown_value_keeps_the_valid_ones() {
        // Diverso dal reset totale di Checkbox/Select: uno sconosciuto in
        // mezzo a valori validi non azzera tutto, solo quello scartato.
        let mut select = make_select(
            &[("I", "Italia"), ("F", "Francia"), ("D", "Germania")],
            None,
            false,
        );
        select.set("I,nonexistent,D");
        assert_eq!(select.get(), "I,D");
    }

    #[test]
    fn set_with_only_unknown_values_deselects_everything() {
        let mut select = make_select(&[("I", "Italia"), ("F", "Francia")], None, false);
        select.set("I"); // seleziona qualcosa prima
        select.set("nonexistent,also_missing");
        assert_eq!(select.get(), "");
    }

    #[test]
    fn get_returns_empty_string_when_nothing_is_selected() {
        let select = make_select(&[("I", "Italia")], None, false);
        assert_eq!(select.get(), "");
    }

    #[test]
    fn selected_values_follows_values_order_not_selection_order() {
        let mut select = make_select(
            &[("I", "Italia"), ("F", "Francia"), ("D", "Germania")],
            None,
            false,
        );
        select.set("D,I"); // "D" passato prima di "I"

        let sel = MultiSelectRef { inner: &select };
        assert_eq!(sel.selected_values().collect::<Vec<_>>(), vec!["I", "D"]);
    }
    #[test]
    fn page_down_moves_by_the_field_height_minus_one() {
        let mut select = make_select(&[("a", "A"), ("b", "B")], Some(0), false); // height: 5
        handle_input_multiselect(
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            &mut select,
        );
        assert_eq!(select.list_state.selected(), Some(4));
    }
}

#[cfg(test)]
mod builder_multiselect_tests {
    use crate::{builder::FormBuilder, error::BuildError};

    #[test]
    fn out_of_range_index_is_ignored_not_clamped() {
        // Comportamento diverso da Select (che clampa all'ultima opzione):
        // qui un indice fuori range viene scartato in silenzio.
        let state = FormBuilder::new()
            .multi_select(1, "Tags")
            .values_ref(&[("a", "A"), ("b", "B")])
            .selected(&[5]) // fuori range: solo indici 0..=1 esistono
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some(String::new()));
    }

    #[test]
    fn field_is_not_dirty_right_after_build() {
        // Regressione sul bug di initial_value disallineato dal formato
        // di get() -- se torna, questo test torna rosso.
        let state = FormBuilder::new()
            .multi_select(1, "Tags")
            .values_ref(&[("a", "A"), ("b", "B"), ("c", "C")])
            .selected(&[0, 2])
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some("a,c".to_owned()));
        assert_eq!(state.is_field_dirty(&1), Some(false));
    }

    #[test]
    fn no_selection_after_selected_wins() {
        let state = FormBuilder::new()
            .multi_select(1, "Tags")
            .values_ref(&[("a", "A"), ("b", "B")])
            .selected(&[0, 1])
            .no_selection() // chiamato per ultimo -> vince questo
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some(String::new()));
    }

    #[test]
    fn value_containing_the_separator_is_rejected_at_build_time() {
        let result = FormBuilder::new()
            .multi_select(1, "Tags")
            .values_ref(&[("a", "A"), ("b,c", "B e C")]) // virgola nel valore
            .build();

        assert_eq!(
            result.err().unwrap(),
            BuildError::InvalidMultiSelectValue { position: 1 }
        );
    }

    #[test]
    fn duplicate_values_are_still_caught() {
        // Non-regressione: il controllo virgola non deve aver soppiantato
        // quello sui duplicati, sono due `if` nello stesso ciclo.
        let result = FormBuilder::new()
            .multi_select(1, "Tags")
            .values_ref(&[("a", "A"), ("a", "A bis")])
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
    fn required_blocks_submit_with_nothing_selected() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut state = FormBuilder::new()
            .multi_select(1, "Tags")
            .values_ref(&[("a", "A"), ("b", "B")])
            .no_selection()
            .required("Select at least one".to_owned())
            .build()
            .unwrap();

        state.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(state.result(), crate::FormResult::Working));
    }

    #[test]
    fn required_allows_submit_with_one_selected() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut state = FormBuilder::new()
            .multi_select(1, "Tags")
            .values_ref(&[("a", "A"), ("b", "B")])
            .selected(&[0])
            .required("Select at least one".to_owned())
            .build()
            .unwrap();

        state.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(state.result(), crate::FormResult::Submitted));
    }

    #[test]
    fn build_with_no_options_does_not_panic() {
        let state = FormBuilder::new()
            .multi_select(1, "Tags")
            .values_ref(&[])
            .optional()
            .build()
            .unwrap();

        assert_eq!(state.value(&1), Some(String::new()));
    }
}

#[cfg(test)]
mod pinnable_tests {
    use ratatui::crossterm::event::KeyModifiers;

    use super::test_helpers::make_select;
    use super::*;

    #[test]
    fn reorder_is_a_no_op_when_not_pinnable() {
        let mut select = make_select(&[("A", "A"), ("B", "B"), ("C", "C")], None, false);
        select.selected = vec![false, true, false];
        select.reorder();
        assert_eq!(select.order, vec![0, 1, 2]);
    }

    #[test]
    fn reorder_pins_selected_items_first_in_original_relative_order() {
        let mut select = make_select(
            &[("A", "A"), ("B", "B"), ("C", "C"), ("D", "D"), ("E", "E")],
            None,
            true,
        );
        select.selected = vec![false, true, false, true, false];
        select.reorder();
        assert_eq!(select.order, vec![1, 3, 0, 2, 4]);
    }

    #[test]
    fn reorder_with_nothing_selected_is_the_identity() {
        let mut select = make_select(&[("A", "A"), ("B", "B")], None, true);
        select.reorder();
        assert_eq!(select.order, vec![0, 1]);
    }

    #[test]
    fn reorder_with_everything_selected_is_also_the_identity() {
        let mut select = make_select(&[("A", "A"), ("B", "B")], None, true);
        select.selected = vec![true, true];
        select.reorder();
        assert_eq!(select.order, vec![0, 1]);
    }

    #[test]
    fn reorder_called_twice_without_changes_is_idempotent() {
        let mut select = make_select(&[("A", "A"), ("B", "B"), ("C", "C")], None, true);
        select.selected = vec![false, true, false];
        select.reorder();
        let first = select.order.clone();
        select.reorder();
        assert_eq!(select.order, first);
    }

    #[test]
    fn toggle_flips_the_original_index_not_the_display_position() {
        let space = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        let mut select = make_select(&[("A", "A"), ("B", "B"), ("C", "C")], None, true);
        select.view = vec![2, 0, 1];
        select.list_state.select(Some(1));

        handle_input_multiselect(space, &mut select);

        assert_eq!(select.selected, vec![true, false, false]);
    }

    #[test]
    fn toggling_the_same_item_twice_returns_to_full_identity_order() {
        let mut select = make_select(&[("A", "A"), ("B", "B"), ("C", "C")], None, true);
        let space = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);

        select.list_state.select(Some(1)); // B
        handle_input_multiselect(space, &mut select); // seleziona B, B va in cima
        assert_eq!(select.order, vec![1, 0, 2]);
        assert_eq!(select.view, vec![1, 0, 2]); // nessuna query attiva: view coincide con order

        select.list_state.select(Some(0)); // cursore ora sulla riga che mostra B
        handle_input_multiselect(space, &mut select); // deseleziona B

        assert_eq!(select.selected, vec![false, false, false]);
        assert_eq!(select.order, vec![0, 1, 2]);
    }

    #[test]
    fn get_is_unaffected_by_a_reordered_view() {
        let mut select = make_select(&[("a", "A"), ("b", "B"), ("c", "C")], None, true);
        select.selected = vec![true, false, true];
        select.reorder();

        assert_eq!(select.get(), "a,c");
    }
}

#[cfg(test)]
mod rebuild_view_tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::test_helpers::make_select;

    use crate::{
        MultiSelectRef,
        widget::{
            common::searchable::Search,
            multi_select::{MultiSelectStatus, handle_input_multiselect},
        },
    };

    fn set_query(select: &mut MultiSelectStatus, query: &str) {
        select.searchable = Search::enabled(select.values.len());

        for c in query.chars() {
            select
                .searchable
                .handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }

        select.refilter();
    }

    #[test]
    fn unselected_item_not_matching_query_is_hidden_even_when_pinnable() {
        let mut select = make_select(&[("A", "Italia"), ("B", "Francia")], None, true);

        set_query(&mut select, "fra");

        assert_eq!(select.view, vec![1]);
    }
    #[test]
    fn selected_item_not_matching_query_stays_visible_when_pinnable() {
        let mut select = make_select(&[("A", "Italia"), ("B", "Francia")], None, true);

        select.selected = vec![true, false];

        set_query(&mut select, "fra");

        assert_eq!(select.view, vec![0, 1]);
    }
    #[test]
    fn selected_item_not_matching_query_is_hidden_when_not_pinnable() {
        let mut select = make_select(&[("A", "Italia"), ("B", "Francia")], None, false);

        select.selected = vec![true, false];

        set_query(&mut select, "fra");

        assert_eq!(select.view, vec![1]);
    }
    #[test]
    fn item_matching_the_query_is_visible_when_not_pinnable() {
        let mut select = make_select(&[("A", "Italia"), ("B", "Francia")], None, false);

        set_query(&mut select, "ita");

        assert_eq!(select.view, vec![0]);
    }
    #[test]
    fn pinned_non_matching_item_has_no_match_positions() {
        let mut select = make_select(&[("A", "Italia"), ("B", "Francia")], None, true);

        select.selected[0] = true;

        set_query(&mut select, "fra");

        assert_eq!(select.view, vec![0, 1]);

        assert_eq!(select.searchable.find_positions(0), &[]);

        assert_eq!(select.searchable.find_positions(1), &[0, 1, 2]);
    }
    #[test]
    fn escape_clears_query_and_restores_full_view() {
        let mut select = make_select(&[("A", "Italia"), ("B", "Francia")], Some(0), false);

        select.searchable = Search::enabled(select.values.len());

        for c in "ita".chars() {
            handle_input_multiselect(
                KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
                &mut select,
            );
        }

        assert_eq!(select.view, vec![0]);

        handle_input_multiselect(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut select);

        assert_eq!(select.searchable.search_query(), Some(""));

        assert_eq!(select.view, vec![0, 1]);
    }
    #[test]
    fn backspace_updates_search_and_restores_matches() {
        let mut select = make_select(&[("A", "Italia"), ("B", "Francia")], Some(0), false);

        select.searchable = Search::enabled(select.values.len());

        for c in "ita".chars() {
            handle_input_multiselect(
                KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
                &mut select,
            );
        }

        assert_eq!(select.view, vec![0]);

        handle_input_multiselect(
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            &mut select,
        );

        assert_eq!(select.searchable.search_query(), Some("it"));
    }
    #[test]
    fn selected_index_returns_original_index_after_reorder() {
        let mut select = make_select(&[("A", "A"), ("B", "B"), ("C", "C")], Some(0), true);

        select.view = vec![2, 0, 1];
        select.list_state.select(Some(0));

        let reference = MultiSelectRef { inner: &select };

        assert_eq!(reference.selected_index(), Some(2));
    }
}
