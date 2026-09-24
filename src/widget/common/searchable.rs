use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::internal::fuzzy::{FuzzyItem, fuzzy_search};

fn all_matches(item_count: usize) -> Vec<SearchMatch> {
    (0..item_count)
        .map(|index| SearchMatch {
            original_index: index,
            positions: Vec::new(),
        })
        .collect()
}

#[derive(Debug)]
pub(crate) struct SearchState {
    length: usize,
    query: String,
    matches: Vec<SearchMatch>,
    visible: Vec<Option<usize>>,
}

impl SearchState {
    fn reset(&mut self) {
        self.query.clear();
        self.matches = all_matches(self.length);
        self.visible = (0..self.length).map(Some).collect();
    }

    fn new(length: usize) -> Self {
        Self {
            length,
            query: "".to_owned(),
            matches: all_matches(length),
            visible: (0..length).map(Some).collect(),
        }
    }

    fn match_position(&self, original_idx: usize) -> Option<usize> {
        self.visible.get(original_idx).copied().flatten()
    }

    fn original_index_at(&self, idx: usize, last_index: usize) -> Option<usize> {
        let last = self.matches.len().saturating_sub(1);
        self.matches
            .get(idx.min(last))
            .map(|matched| matched.original_index.min(last_index))
    }

    fn filtered_index_of(&self, original: Option<usize>) -> Option<usize> {
        original.and_then(|idx| self.match_position(idx))
    }

    fn refilter<'a>(&mut self, iter: impl Iterator<Item = &'a str>) {
        self.visible = (0..self.length).map(|_| None).collect();
        let filtered = fuzzy_search(iter, self.query.as_str());
        self.matches = filtered
            .into_iter()
            .enumerate()
            .map(|(pos, item)| {
                self.visible[item.index] = Some(pos);
                item.into()
            })
            .collect();
    }

    fn has_query(&self) -> bool {
        !self.query.is_empty()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (usize, &[usize])> + '_> {
        Box::new(
            self.matches
                .iter()
                .map(|m| (m.original_index, m.positions.as_slice())),
        )
    }

    fn contains_original_index(&self, original_idx: usize) -> bool {
        self.match_position(original_idx).is_some()
    }

    fn find_position(&self, original_idx: usize) -> &[usize] {
        self.match_position(original_idx)
            .and_then(|pos| self.matches.get(pos))
            .map(|m| m.positions.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Debug)]
struct SearchMatch {
    original_index: usize,
    positions: Vec<usize>,
}

impl<'a> From<FuzzyItem<'a>> for SearchMatch {
    fn from(value: FuzzyItem) -> Self {
        Self {
            original_index: value.index,
            positions: value.positions,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) enum Search {
    #[default]
    Disabled,
    Enabled(SearchState),
}

impl Search {
    pub(crate) fn enabled(len: usize) -> Self {
        Self::Enabled(SearchState::new(len))
    }

    pub(crate) fn reset(&mut self) -> bool {
        if let Self::Enabled(filtered) = self {
            filtered.reset();
            true
        } else {
            false
        }
    }

    pub(crate) fn original_index_at(&self, idx: usize, item_count: usize) -> Option<usize> {
        if item_count == 0 {
            return None;
        }
        let last_index = item_count - 1;
        if let Self::Enabled(filtered) = self {
            filtered.original_index_at(idx, last_index)
        } else {
            Some(idx.min(last_index))
        }
    }

    pub(crate) fn filtered_index_of(&self, original: Option<usize>) -> Option<usize> {
        match self {
            Self::Enabled(state) => state.filtered_index_of(original),
            Self::Disabled => original,
        }
    }

    pub(crate) fn refilter<'a>(&mut self, iter: impl Iterator<Item = &'a str>) {
        if let Self::Enabled(filtered) = self {
            filtered.refilter(iter);
        }
    }

    pub(crate) fn has_query(&self) -> bool {
        if let Self::Enabled(filtered) = self {
            filtered.has_query()
        } else {
            false
        }
    }
    pub(crate) fn has_filter(&self) -> bool {
        matches!(self, Self::Enabled(_))
    }

    /// Updates the search query only.
    /// The caller must invoke `refilter()` when this returns `true`.
    pub(crate) fn handle_input(&mut self, key_event: KeyEvent) -> bool {
        if let Self::Enabled(filtered) = self {
            match key_event.code {
                KeyCode::Char(c) => {
                    filtered.query.push(c);
                    true
                }
                KeyCode::Backspace if !filtered.query.is_empty() => {
                    filtered.query.pop();
                    true
                }
                KeyCode::Esc if !filtered.query.is_empty() => {
                    filtered.query.clear();
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub(crate) fn iter(&self, length: usize) -> Box<dyn Iterator<Item = (usize, &[usize])> + '_> {
        if let Self::Enabled(filtered) = self {
            filtered.iter()
        } else {
            Box::new((0..length).map(|i| (i, &[] as &[usize])))
        }
    }

    pub(crate) fn search_query(&self) -> Option<&str> {
        if let Self::Enabled(filtered) = self {
            Some(filtered.query.as_str())
        } else {
            None
        }
    }

    pub(crate) fn filtered_count(&self, item_count: usize) -> usize {
        match self {
            Search::Disabled => item_count,
            Search::Enabled(search_state) => search_state.matches.len(),
        }
    }

    pub(crate) fn contains_original_index(&self, original_idx: usize) -> bool {
        match self {
            Search::Disabled => true,
            Search::Enabled(search_state) => search_state.contains_original_index(original_idx),
        }
    }

    pub(crate) fn find_positions(&self, original_idx: usize) -> &[usize] {
        match self {
            Search::Disabled => &[],
            Search::Enabled(search_state) => search_state.find_position(original_idx),
        }
    }
}

#[cfg(test)]
mod searchable_test {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn index_mapping_round_trips() {
        let mut search = Search::enabled(4);

        for c in "ap".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }

        search.refilter(["apple", "banana", "apricot", "pear"].into_iter());

        for filtered_index in 0..2 {
            let original = search.original_index_at(filtered_index, 3).unwrap();

            assert_eq!(
                search.filtered_index_of(Some(original)),
                Some(filtered_index)
            );
        }
    }
    #[test]
    fn disabled_search_maps_indices_directly() {
        let search = Search::default();

        assert_eq!(search.original_index_at(2, 10), Some(2));
        assert_eq!(search.filtered_index_of(Some(2)), Some(2));
        assert_eq!(search.filtered_index_of(None), None);
    }

    #[test]
    fn enabled_search_maps_filtered_to_original_index() {
        let mut search = Search::enabled(4);

        for c in "ap".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }

        search.refilter(["apple", "banana", "apricot", "pear"].into_iter());

        assert_eq!(search.original_index_at(0, 3), Some(0));
        assert_eq!(search.original_index_at(1, 3), Some(2));
    }

    #[test]
    fn enabled_search_maps_original_to_filtered_index() {
        let mut search = Search::enabled(4);

        for c in "ap".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }

        search.refilter(["apple", "banana", "apricot", "pear"].into_iter());

        assert_eq!(search.filtered_index_of(Some(0)), Some(0));
        assert_eq!(search.filtered_index_of(Some(2)), Some(1));
        assert_eq!(search.filtered_index_of(Some(1)), None);
    }

    #[test]
    fn escape_on_empty_query_is_not_handled() {
        let mut search = Search::enabled(3);

        assert!(!search.handle_input(key(KeyCode::Esc)));
    }

    #[test]
    fn escape_on_non_empty_query_clears_it() {
        let mut search = Search::enabled(3);

        search.handle_input(key(KeyCode::Char('a')));

        assert!(search.handle_input(key(KeyCode::Esc)));
        assert_eq!(search.search_query(), Some(""));
    }

    #[test]
    fn backspace_on_empty_query_is_not_handled() {
        let mut search = Search::enabled(3);

        assert!(!search.handle_input(key(KeyCode::Backspace)));
    }

    #[test]
    fn reset_restores_identity_mapping() {
        let mut search = Search::enabled(3);

        search.handle_input(key(KeyCode::Char('z')));
        search.refilter(["apple", "banana", "pear"].into_iter());

        assert_eq!(search.iter(3).count(), 0);

        search.reset();

        let indices = search.iter(3).map(|(index, _)| index).collect::<Vec<_>>();

        assert_eq!(indices, vec![0, 1, 2]);
    }
    #[test]
    fn refilter_updates_matches_for_current_query() {
        let mut search = Search::enabled(3);

        search.handle_input(key(KeyCode::Char('a')));
        search.refilter(["apple", "pear", "kiwi"].into_iter());

        let indices = search.iter(3).map(|(index, _)| index).collect::<Vec<_>>();

        assert_eq!(indices, vec![0, 1]);
    }
    #[test]
    fn handle_input_changes_query_but_does_not_refilter() {
        let mut search = Search::enabled(3);

        search.handle_input(key(KeyCode::Char('a')));

        assert_eq!(search.search_query(), Some("a"));

        let indices = search.iter(3).map(|(index, _)| index).collect::<Vec<_>>();

        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn original_index_at_maps_filtered_position_to_original_index() {
        let search = Search::Enabled(SearchState {
            length: 3,
            query: "x".to_owned(),
            matches: vec![
                SearchMatch {
                    original_index: 2,
                    positions: vec![],
                },
                SearchMatch {
                    original_index: 0,
                    positions: vec![],
                },
            ],
            visible: vec![],
        });

        assert_eq!(search.original_index_at(1, 3), Some(0));
    }
    #[test]
    fn original_index_at_returns_none_when_there_are_no_matches() {
        let search = Search::Enabled(SearchState {
            length: 3,
            query: "zzz".to_owned(),
            matches: vec![],
            visible: vec![],
        });

        assert_eq!(search.original_index_at(0, 3), None);
    }
    #[test]
    fn original_index_at_clamps_visible_index_to_last_match() {
        let search = Search::Enabled(SearchState {
            length: 3,
            query: "x".to_owned(),
            matches: vec![
                SearchMatch {
                    original_index: 0,
                    positions: vec![],
                },
                SearchMatch {
                    original_index: 2,
                    positions: vec![],
                },
            ],
            visible: vec![],
        });

        assert_eq!(search.original_index_at(usize::MAX, 3), Some(2));
    }
    #[test]
    fn refilter_with_matching_query_narrows_matches() {
        let mut search = Search::enabled(3);

        search.handle_input(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        search.handle_input(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        search.handle_input(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));

        search.refilter(["Italia", "Francia", "Germania"].into_iter());

        let result = search
            .iter(3)
            .map(|(index, positions)| (index, positions.to_vec()))
            .collect::<Vec<_>>();

        assert_eq!(result, vec![(2, vec![0, 1, 2])]);
    }
    #[test]
    fn refilter_with_empty_query_restores_all_items() {
        let mut search = Search::enabled(3);

        search.refilter(["Italia", "Francia", "Germania"].into_iter());

        let result = search
            .iter(3)
            .map(|(index, positions)| (index, positions.to_vec()))
            .collect::<Vec<_>>();

        assert_eq!(result, vec![(0, vec![]), (1, vec![]), (2, vec![]),]);
    }
    #[test]
    fn reset_restores_membership_and_clears_match_positions() {
        let mut search = Search::enabled(3);

        for c in "ger".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }

        search.refilter(["Italia", "Francia", "Germania"].into_iter());

        search.reset();

        assert!(search.contains_original_index(0));
        assert!(search.contains_original_index(1));
        assert!(search.contains_original_index(2));

        assert_eq!(search.find_positions(0), &[]);
        assert_eq!(search.find_positions(1), &[]);
        assert_eq!(search.find_positions(2), &[]);
    }
    #[test]
    fn filtered_count_returns_item_count_when_search_is_disabled() {
        let search = Search::Disabled;

        assert_eq!(search.filtered_count(4), 4);
    }
    #[test]
    fn filtered_count_returns_number_of_matches() {
        let mut search = Search::enabled(4);

        for c in "ap".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }

        search.refilter(["apple", "banana", "apricot", "pear"].into_iter());

        assert_eq!(search.filtered_count(4), 2);
    }
    #[test]
    fn disabled_search_has_no_match_positions() {
        let search = Search::Disabled;

        assert_eq!(search.find_positions(0), &[]);
    }
    #[test]
    fn find_positions_returns_empty_slice_for_non_matching_item() {
        let mut search = Search::enabled(3);

        for c in "ger".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }

        search.refilter(["Italia", "Francia", "Germania"].into_iter());

        assert_eq!(search.find_positions(0), &[]);
    }
    #[test]
    fn find_positions_returns_match_positions_for_original_index() {
        let mut search = Search::enabled(3);

        for c in "ger".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }

        search.refilter(["Italia", "Francia", "Germania"].into_iter());

        assert_eq!(search.find_positions(2), &[0, 1, 2]);
    }
    #[test]
    fn disabled_search_considers_every_original_index_visible() {
        let search = Search::Disabled;

        assert!(search.contains_original_index(0));
        assert!(search.contains_original_index(42));
    }
    #[test]
    fn contains_original_index_reflects_current_filter() {
        let mut search = Search::enabled(4);

        for c in "ap".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }

        search.refilter(["apple", "banana", "apricot", "pear"].into_iter());

        assert!(search.contains_original_index(0));
        assert!(!search.contains_original_index(1));
        assert!(search.contains_original_index(2));
        assert!(!search.contains_original_index(3));
    }
    #[test]
    fn filtered_index_of_an_out_of_range_index_is_none() {
        let search = Search::enabled(3);

        assert_eq!(search.filtered_index_of(Some(10)), None);
    }

    #[test]
    fn index_mapping_follows_score_order_not_original_order() {
        // "xaxb" matches "ab" at [1, 3] -> score 1 + (10 - 2) = 9
        // "ab"   matches "ab" at [0, 1] -> score 1 + 20      = 21
        // So the item with the higher original index comes first in `matches`.
        let items = ["xaxb", "ab"];
        let mut search = Search::enabled(items.len());

        for c in "ab".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }
        search.refilter(items.into_iter());

        assert_eq!(search.original_index_at(0, 2), Some(1));
        assert_eq!(search.original_index_at(1, 2), Some(0));

        assert_eq!(search.filtered_index_of(Some(1)), Some(0));
        assert_eq!(search.filtered_index_of(Some(0)), Some(1));

        assert_eq!(search.find_positions(0), &[1, 3]);
        assert_eq!(search.find_positions(1), &[0, 1]);
    }

    #[test]
    fn a_second_refilter_forgets_items_that_no_longer_match() {
        let items = ["apple", "banana", "apricot", "pear"];
        let mut search = Search::enabled(items.len());

        for c in "ap".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }
        search.refilter(items.into_iter()); // apple, apricot

        search.handle_input(key(KeyCode::Char('r')));
        search.refilter(items.into_iter()); // only apricot

        assert!(!search.contains_original_index(0));
        assert_eq!(search.filtered_index_of(Some(0)), None);
        assert_eq!(search.find_positions(0), &[]);

        assert!(search.contains_original_index(2));
        assert_eq!(search.filtered_index_of(Some(2)), Some(0));
        assert_eq!(search.find_positions(2), &[0, 1, 2]);
    }

    #[test]
    fn reset_restores_identity_index_mapping() {
        let items = ["apple", "banana", "apricot", "pear"];
        let mut search = Search::enabled(items.len());

        for c in "apr".chars() {
            search.handle_input(key(KeyCode::Char(c)));
        }
        search.refilter(items.into_iter());

        search.reset();

        for i in 0..items.len() {
            assert_eq!(search.filtered_index_of(Some(i)), Some(i));
        }
    }
}
