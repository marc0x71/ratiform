use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    widgets::ListState,
};

use crate::internal::list::HorizontalListState;

// DIRECTION
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub(crate) enum StateDirection {
    Horizontal(HorizontalListState),
    Vertical(ListState),
}

impl StateDirection {
    pub(crate) fn new(direction: Direction, selected: Option<usize>) -> Self {
        match direction {
            Direction::Horizontal => {
                StateDirection::Horizontal(HorizontalListState::default().with_selected(selected))
            }
            Direction::Vertical => {
                StateDirection::Vertical(ListState::default().with_selected(selected))
            }
        }
    }

    pub(crate) fn selected(&self) -> Option<usize> {
        match self {
            StateDirection::Horizontal(state) => state.selected(),
            StateDirection::Vertical(state) => state.selected(),
        }
    }

    pub(crate) fn select(&mut self, index: Option<usize>) {
        match self {
            StateDirection::Horizontal(state) => state.select(index),
            StateDirection::Vertical(state) => state.select(index),
        }
    }

    fn scroll_up_by(&mut self, amount: u16) {
        if let StateDirection::Vertical(state) = self {
            state.scroll_up_by(amount);
        }
    }

    fn scroll_down_by(&mut self, amount: u16) {
        if let StateDirection::Vertical(state) = self {
            state.scroll_down_by(amount);
        }
    }

    fn right(&mut self) {
        if let StateDirection::Horizontal(state) = self {
            state.select_next();
        }
    }

    fn left(&mut self) {
        if let StateDirection::Horizontal(state) = self {
            state.select_previous();
        }
    }

    pub(crate) fn up(&mut self) {
        if let StateDirection::Vertical(state) = self {
            state.select_previous();
        }
    }

    pub(crate) fn down(&mut self) {
        if let StateDirection::Vertical(state) = self {
            state.select_next();
        }
    }

    fn home(&mut self) {
        match self {
            StateDirection::Horizontal(state) => state.select_first(),
            StateDirection::Vertical(state) => state.select_first(),
        }
    }

    fn end(&mut self) {
        match self {
            StateDirection::Horizontal(state) => state.select_last(),
            StateDirection::Vertical(state) => state.select_last(),
        }
    }

    pub(crate) fn handle_input(&mut self, key_event: KeyEvent, view_height: u16) {
        match key_event.code {
            KeyCode::Left => self.left(),
            KeyCode::Right => self.right(),
            KeyCode::Up => self.up(),
            KeyCode::Down => self.down(),
            KeyCode::Home => self.home(),
            KeyCode::End => self.end(),
            KeyCode::PageUp => self.scroll_up_by(view_height.saturating_sub(1).max(1)),
            KeyCode::PageDown => self.scroll_down_by(view_height.saturating_sub(1).max(1)),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn vertical(selected: usize) -> StateDirection {
        StateDirection::Vertical(ListState::default().with_selected(Some(selected)))
    }

    fn horizontal(selected: usize) -> StateDirection {
        StateDirection::Horizontal(HorizontalListState::default().with_selected(Some(selected)))
    }

    fn press(cursor: &mut StateDirection, code: KeyCode, view_height: u16) {
        cursor.handle_input(KeyEvent::new(code, KeyModifiers::NONE), view_height);
    }

    // ---------- an arrow only works along its own axis ----------

    #[test]
    fn vertical_ignores_left_and_right() {
        let mut cursor = vertical(2);
        press(&mut cursor, KeyCode::Left, 5);
        press(&mut cursor, KeyCode::Right, 5);
        assert_eq!(cursor.selected(), Some(2));
    }

    #[test]
    fn horizontal_ignores_up_and_down() {
        let mut cursor = horizontal(2);
        press(&mut cursor, KeyCode::Up, 5);
        press(&mut cursor, KeyCode::Down, 5);
        assert_eq!(cursor.selected(), Some(2));
    }

    // ---------- single-step movement ----------

    #[test]
    fn vertical_moves_one_item_with_up_and_down() {
        let mut cursor = vertical(2);
        press(&mut cursor, KeyCode::Down, 5);
        assert_eq!(cursor.selected(), Some(3));
        press(&mut cursor, KeyCode::Up, 5);
        press(&mut cursor, KeyCode::Up, 5);
        assert_eq!(cursor.selected(), Some(1));
    }

    #[test]
    fn horizontal_moves_one_item_with_left_and_right() {
        let mut cursor = horizontal(2);
        press(&mut cursor, KeyCode::Right, 5);
        assert_eq!(cursor.selected(), Some(3));
        press(&mut cursor, KeyCode::Left, 5);
        press(&mut cursor, KeyCode::Left, 5);
        assert_eq!(cursor.selected(), Some(1));
    }

    #[test]
    fn up_on_the_first_item_stays_on_the_first_item() {
        let mut cursor = vertical(0);
        press(&mut cursor, KeyCode::Up, 5);
        assert_eq!(cursor.selected(), Some(0));
    }

    #[test]
    fn left_on_the_first_item_stays_on_the_first_item() {
        let mut cursor = horizontal(0);
        press(&mut cursor, KeyCode::Left, 5);
        assert_eq!(cursor.selected(), Some(0));
    }

    // ---------- Home / End: both orientations ----------

    #[test]
    fn home_jumps_to_the_first_item_in_both_orientations() {
        for mut cursor in [vertical(5), horizontal(5)] {
            press(&mut cursor, KeyCode::Home, 5);
            assert_eq!(cursor.selected(), Some(0));
        }
    }

    #[test]
    fn end_leaves_the_first_item_in_both_orientations() {
        for mut cursor in [vertical(0), horizontal(0)] {
            press(&mut cursor, KeyCode::End, 5);
            assert_ne!(cursor.selected(), Some(0));
        }
    }

    // ---------- PageUp / PageDown: vertical, page = height - 1 (at least 1) ----------

    #[test]
    fn page_down_moves_by_height_minus_one() {
        let mut cursor = vertical(1);
        press(&mut cursor, KeyCode::PageDown, 4);
        assert_eq!(cursor.selected(), Some(4));
    }

    #[test]
    fn page_up_moves_back_by_height_minus_one() {
        let mut cursor = vertical(5);
        press(&mut cursor, KeyCode::PageUp, 4);
        assert_eq!(cursor.selected(), Some(2));
    }

    #[test]
    fn page_up_does_not_go_below_the_first_item() {
        let mut cursor = vertical(1);
        press(&mut cursor, KeyCode::PageUp, 4);
        assert_eq!(cursor.selected(), Some(0));
    }

    #[test]
    fn a_one_row_view_pages_by_one_item() {
        let mut cursor = vertical(3);
        press(&mut cursor, KeyCode::PageDown, 1);
        assert_eq!(cursor.selected(), Some(4));
        press(&mut cursor, KeyCode::PageUp, 1);
        press(&mut cursor, KeyCode::PageUp, 1);
        assert_eq!(cursor.selected(), Some(2));
    }

    #[test]
    fn a_zero_height_view_neither_panics_nor_freezes() {
        let mut cursor = vertical(0);
        press(&mut cursor, KeyCode::PageDown, 0);
        assert_eq!(cursor.selected(), Some(1));
    }

    // ---------- PageUp / PageDown: no effect when horizontal ----------

    #[test]
    fn horizontal_ignores_page_up_and_page_down() {
        for view_height in [1, 2, 5] {
            let mut cursor = horizontal(2);
            press(&mut cursor, KeyCode::PageDown, view_height);
            press(&mut cursor, KeyCode::PageUp, view_height);
            assert_eq!(cursor.selected(), Some(2), "height {view_height}");
        }
    }

    // ---------- everything else is not our business ----------

    #[test]
    fn unmapped_keys_leave_the_selection_untouched() {
        let keys = [
            KeyCode::Char('x'),
            KeyCode::Char(' '),
            KeyCode::Enter,
            KeyCode::Backspace,
            KeyCode::Delete,
        ];
        for code in keys {
            for mut cursor in [vertical(2), horizontal(2)] {
                press(&mut cursor, code, 5);
                assert_eq!(cursor.selected(), Some(2), "{code:?} moved the cursor");
            }
        }
    }
}
