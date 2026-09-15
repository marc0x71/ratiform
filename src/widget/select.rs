use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};

use crate::{
    FormState,
    builder::FormBuilder,
    error::BuildError,
    field::{Field, FieldKind, FieldOptions},
    field_builder_common,
    internal::{
        fuzzy::fuzzy_search,
        list::{HorizontalList, HorizontalListState},
    },
    style::{FormStyle, Parts, States, Widgets},
};

pub(crate) enum SelectDirection {
    Horizontal,
    Vertical,
}

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
/// single scrolling row navigated with `Left`/`Right` instead.
pub struct SelectBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: Option<usize>,
    pub(crate) options: FieldOptions,
    pub(crate) highlight_symbol: String,
    pub(crate) direction: SelectDirection,
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
    /// navigated with `Left`/`Right`. `Up`/`Down`/`Home`/`End`/`PageUp`/
    /// `PageDown` become no-ops.
    pub fn horizontal(mut self) -> Self {
        self.direction = SelectDirection::Horizontal;
        self
    }

    /// Lays the options out in a column — the default. Reverses
    /// [`horizontal`](Self::horizontal).
    pub fn vertical(mut self) -> Self {
        self.direction = SelectDirection::Vertical;
        self
    }

    /// Enables or disables scrollbar for this select.
    pub fn scrollbar(mut self, show_scrollbar: bool) -> Self {
        self.scrollbar = show_scrollbar;
        self
    }

    /// Enables searchable feature for this select.
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
        let list_state = match self.direction {
            SelectDirection::Horizontal => SelectStateDirection::Horizontal(
                HorizontalListState::default().with_selected(self.selected),
            ),
            SelectDirection::Vertical => {
                SelectStateDirection::Vertical(ListState::default().with_selected(self.selected))
            }
        };
        if self.scrollbar && matches!(self.direction, SelectDirection::Horizontal) {
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
                    Some("".to_string())
                } else {
                    None
                },
                filtered: (0..len).map(|i| (i, vec![])).collect(),
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
    pub(crate) spacing: usize,
    pub(crate) preview: usize,
    pub(crate) scrollbar: Option<ScrollbarState>,
    pub(crate) searchable: Option<String>,
    pub(crate) filtered: Vec<(usize, Vec<usize>)>,
}

impl SelectStatus {
    fn selected(&self) -> Option<usize> {
        let last_values = self.values.len().saturating_sub(1);
        let last_filtered = self.filtered.len().saturating_sub(1);
        self.list_state.selected().and_then(|filtered_idx| {
            self.filtered
                .get(filtered_idx.min(last_filtered))
                .map(|value_idx| value_idx.0.min(last_values))
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
        let original = self.values.iter().position(|(k, _)| k == value);
        let filtered_pos = original.and_then(|orig_idx| {
            self.filtered
                .iter()
                .position(|(filter_idx, _)| *filter_idx == orig_idx)
        });
        self.list_state.select(filtered_pos);
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

    fn refilter(&mut self) {
        let values = self
            .values
            .iter()
            .map(|(_, v)| v.as_str())
            .collect::<Vec<_>>();
        let filtered = fuzzy_search(&values, self.searchable.as_ref().map_or("", |v| v));
        self.filtered = filtered
            .into_iter()
            .map(|f| (f.index, f.positions))
            .collect();
    }

    pub(crate) fn special_key_handled(&self) -> Vec<KeyCode> {
        if self.searchable.as_ref().is_some_and(|q| !q.is_empty()) {
            vec![KeyCode::Esc]
        } else {
            Vec::new()
        }
    }
}

// EVENT
pub(crate) fn handle_input_select(key_event: KeyEvent, select: &mut SelectStatus) {
    let mut need_refilter = false;
    match key_event.code {
        KeyCode::Left => select.left(),
        KeyCode::Right => select.right(),
        KeyCode::Up => select.up(),
        KeyCode::Down => select.down(),
        KeyCode::Home => select.home(),
        KeyCode::End => select.end(),
        KeyCode::PageUp => select.page_up(),
        KeyCode::PageDown => select.page_down(),
        KeyCode::Char(c) => {
            if let Some(ref mut query) = select.searchable {
                query.push(c);
                need_refilter = true;
            }
        }
        KeyCode::Backspace => {
            if let Some(ref mut query) = select.searchable {
                need_refilter = true;
                let _ = query.pop();
            }
        }
        KeyCode::Esc => {
            if let Some(ref mut query) = select.searchable {
                need_refilter = true;
                query.clear();
            }
        }
        _ => {}
    }
    if need_refilter {
        select.refilter();
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
    let list_area = if select.scrollbar.is_some()
        && matches!(select.list_state, SelectStateDirection::Vertical(_))
    {
        Rect {
            width: area.width.saturating_sub(1),
            ..area
        }
    } else {
        area
    };

    let filtered = select
        .filtered
        .iter()
        .filter_map(|(idx, pos)| select.values.get(*idx).map(|(_, v)| (v, pos)))
        .collect::<Vec<_>>();

    let normal = style.get(Widgets::SELECT, Parts::ITEM, field_state);
    let highlight = style.get(Widgets::SELECT, Parts::MATCH, field_state);
    let items: Vec<Line<'_>> = filtered
        .iter()
        .map(|item| Line::from(make_spans(item.0, item.1, normal, highlight)))
        .collect();

    match select.list_state {
        SelectStateDirection::Horizontal(ref mut list_state) => {
            let list = HorizontalList::new(items, select.spacing, select.preview)
                .highlight_style(style.get(Widgets::SELECT, Parts::ACTIVE, field_state));

            StatefulWidget::render(list, list_area, buf, list_state);
        }
        SelectStateDirection::Vertical(ref mut list_state) => {
            let list = List::new(items)
                .highlight_style(style.get(Widgets::SELECT, Parts::ACTIVE, field_state))
                .highlight_symbol(select.highlight_symbol.as_str());
            StatefulWidget::render(list, list_area, buf, list_state);
        }
    }

    if let Some(ref mut scroll_state) = select.scrollbar {
        let (orientation, selected, [begin_sym, end_sym]) = match &select.list_state {
            SelectStateDirection::Horizontal(list_state) => (
                ScrollbarOrientation::HorizontalBottom,
                list_state.selected().unwrap_or_default(),
                ["←", "→"],
            ),
            SelectStateDirection::Vertical(list_state) => (
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
            list_state: match selected {
                Some(idx) => {
                    SelectStateDirection::Vertical(ListState::default().with_selected(Some(idx)))
                }
                None => SelectStateDirection::Vertical(ListState::default()),
            },
            height: 5,
            highlight_symbol: "> ".to_string(),
            spacing: 2,
            preview: 2,
            scrollbar: None,
            searchable: None,
            filtered: (0..values.len()).map(|i| (i, vec![])).collect(),
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
    fn selected_resolves_filtered_position_to_original_index() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );
        // simulo un filtro che ha riordinato: posizione 0 della vista
        // filtrata punta all'originale 2, posizione 1 punta all'originale 0
        select.filtered = vec![(2, vec![]), (0, vec![])];
        select.list_state.select(Some(1));

        assert_eq!(select.selected(), Some(0)); // indice ORIGINALE, non 1
    }

    #[test]
    fn selected_is_none_when_filtered_is_empty() {
        let mut select = make_select(&[("IT", "Italia"), ("FR", "Francia")], None);
        select.filtered = vec![]; // zero match
        select.list_state.select(Some(0));

        assert_eq!(select.selected(), None);
    }

    #[test]
    fn selected_clamps_out_of_range_list_state_to_last_filtered_entry() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );
        select.list_state.select(Some(usize::MAX));
        assert_eq!(select.selected(), Some(2));
    }

    #[test]
    fn refilter_with_empty_query_restores_full_list_in_original_order() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );
        select.searchable = Some(String::new());

        select.refilter();

        assert_eq!(select.filtered, vec![(0, vec![]), (1, vec![]), (2, vec![])]);
    }

    #[test]
    fn refilter_with_matching_query_narrows_and_scores() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );
        select.searchable = Some("ger".to_owned());

        select.refilter();

        assert_eq!(select.filtered, vec![(2, vec![0, 1, 2])]);
    }

    #[test]
    fn refilter_with_no_matches_empties_filtered() {
        let mut select = make_select(&[("IT", "Italia"), ("FR", "Francia")], None);
        select.searchable = Some("xyz".to_owned());

        select.refilter();

        assert_eq!(select.filtered, vec![]);
    }

    #[test]
    fn refilter_resets_cursor_to_first_match_not_to_stale_position() {
        let mut select = make_select(
            &[("IT", "Italia"), ("FR", "Francia"), ("DE", "Germania")],
            None,
        );
        select.list_state.select(Some(2)); // cursore su "Germania"

        select.searchable = Some("fra".to_owned()); // ora l'utente digita
        select.refilter();

        assert_eq!(select.selected(), Some(1)); // indice originale di Francia
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
