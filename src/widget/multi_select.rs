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

/// Builder for a multi-select field: any number of options can be checked
/// with `Space`, navigated with the arrow keys. Started with
/// [`FormBuilder::multi_select`](crate::builder::FormBuilder::multi_select).
/// Like the other field builders, it supports the common options
/// `required`, `optional`, `disabled`, `readonly`, `height`, `validator`,
/// and `normalizer`.
///
/// Cursor and selection are independent here, unlike `Select` — moving
/// the cursor never changes what's selected.
pub struct MultiSelectBuilder<T> {
    pub(crate) id: T,
    pub(crate) form: FormBuilder<T>,
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: Vec<usize>,
    pub(crate) options: FieldOptions,
    pub(crate) selected_symbol: String,
    pub(crate) unselected_symbol: String,
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

        self.form.push_field(Field {
            id: self.id,
            kind: FieldKind::MultiSelect(MultiSelectStatus {
                label: self.label,
                values: self.values,
                list_state: ListState::default().with_selected(Some(0)),
                height: self.options.height,
                selected_symbol: self.selected_symbol,
                unselected_symbol: self.unselected_symbol,
                selected,
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
        self.inner.list_state.selected()
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
}

// STATUS
#[derive(Debug)]
pub struct MultiSelectStatus {
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) selected: Vec<bool>,
    pub(crate) list_state: ListState,
    pub(crate) height: u16,
    pub(crate) selected_symbol: String,
    pub(crate) unselected_symbol: String,
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

    fn toggle(&mut self) {
        if let Some(pos) = self.list_state.selected()
            && let Some(sel) = self.selected.get_mut(pos)
        {
            *sel = !*sel;
        }
    }
}

// EVENT
pub(crate) fn handle_input_multiselect(key_event: KeyEvent, select: &mut MultiSelectStatus) {
    match key_event.code {
        KeyCode::Up => select.up(),
        KeyCode::Down => select.down(),
        KeyCode::Home => select.home(),
        KeyCode::End => select.end(),
        KeyCode::PageUp => select.page_up(),
        KeyCode::PageDown => select.page_down(),
        KeyCode::Char(' ') => select.toggle(),
        _ => {}
    }
}

// RENDER
pub(crate) fn render_multiselect(
    area: Rect,
    buf: &mut Buffer,
    select: &mut MultiSelectStatus,
    value_style: Style,
    highlight_style: Style,
) -> Option<(u16, u16)> {
    let mut items = Vec::new();
    for (idx, (_, v)) in select.values.iter().enumerate() {
        let prefix = if select.selected[idx] {
            select.selected_symbol.as_str()
        } else {
            select.unselected_symbol.as_str()
        };
        items.push(format!("{prefix}{v}"));
    }

    let list = List::new(items)
        .style(value_style)
        .highlight_style(highlight_style);

    StatefulWidget::render(list, area, buf, &mut select.list_state);

    None
}

#[cfg(test)]
mod multiselect_toggle_test {
    use super::*;

    fn make_select(values: &[(&str, &str)], selected: Option<usize>) -> MultiSelectStatus {
        MultiSelectStatus {
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
            selected_symbol: "> ".to_string(),
            unselected_symbol: "  ".to_string(),
            selected: vec![false; values.len()],
        }
    }

    #[test]
    fn down_poi_toggle_senza_mai_renderizzare() {
        let mut select = make_select(&[("a", "A"), ("b", "B")], None);

        select.list_state.select_next(); // simula il tasto Down
        select.toggle(); // simula il tasto Space

        // cosa ti aspetti qui?
    }

    #[test]
    fn up_poi_toggle_senza_mai_renderizzare() {
        let mut select = make_select(&[("a", "A"), ("b", "B")], None);

        select.list_state.select_previous(); // Up, non Down
        select.toggle();

        // e ora?
    }

    #[test]
    fn toggle_dopo_up_senza_mai_renderizzare_non_va_in_panic() {
        // ListState::select_previous() partendo da None usa usize::MAX come
        // sentinella finché il widget non è mai stato renderizzato (Ratatui
        // non conosce ancora la lunghezza della lista) -- toggle() deve
        // reggere un indice del genere senza panicare.
        let mut select = make_select(&[("a", "A"), ("b", "B")], None);

        select.list_state.select_previous();
        select.toggle(); // non deve panicare

        assert_eq!(select.get(), ""); // e non deve aver selezionato nulla
    }
}

#[cfg(test)]
mod multiselect_test {
    use super::*;

    fn make_select(values: &[(&str, &str)], selected: Option<usize>) -> MultiSelectStatus {
        MultiSelectStatus {
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
            selected_symbol: "> ".to_string(),
            unselected_symbol: "  ".to_string(),
            selected: vec![false; values.len()],
        }
    }

    #[test]
    fn set_then_get_round_trips_in_values_order_not_input_order() {
        // "F,I" in ingresso, ma "I" viene prima di "F" nella lista -- get()
        // deve seguire l'ordine di `values`, non l'ordine passato a set().
        let mut select = make_select(
            &[("I", "Italia"), ("F", "Francia"), ("D", "Germania")],
            None,
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
        );
        select.set("I,nonexistent,D");
        assert_eq!(select.get(), "I,D");
    }

    #[test]
    fn set_with_only_unknown_values_deselects_everything() {
        let mut select = make_select(&[("I", "Italia"), ("F", "Francia")], None);
        select.set("I"); // seleziona qualcosa prima
        select.set("nonexistent,also_missing");
        assert_eq!(select.get(), "");
    }

    #[test]
    fn get_returns_empty_string_when_nothing_is_selected() {
        let select = make_select(&[("I", "Italia")], None);
        assert_eq!(select.get(), "");
    }

    #[test]
    fn selected_values_follows_values_order_not_selection_order() {
        let mut select = make_select(
            &[("I", "Italia"), ("F", "Francia"), ("D", "Germania")],
            None,
        );
        select.set("D,I"); // "D" passato prima di "I"

        let sel = MultiSelectRef { inner: &select };
        assert_eq!(sel.selected_values().collect::<Vec<_>>(), vec!["I", "D"]);
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
