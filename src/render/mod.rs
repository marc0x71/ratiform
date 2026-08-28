use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget};

use crate::{
    Form, FormLayout, FormState,
    field::{Field, FieldKind},
    render::{
        custom::{render_custom, required_height_custom},
        horizontal::{render_horizontal, required_height_horizontal},
        stacked::{render_stacked, required_height_stacked},
    },
    style::FormStyle,
    widget::{
        check_box::render_checkbox, select::render_select, single_line::render_singleline,
        text_area::render_textarea,
    },
};

mod custom;
mod horizontal;
mod stacked;

impl<T: PartialEq> StatefulWidget for Form<T> {
    type State = FormState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_form(&self.layout, &self.style, area, buf, state);
    }
}

impl<T: PartialEq> StatefulWidget for &Form<T> {
    type State = FormState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_form(&self.layout, &self.style, area, buf, state);
    }
}

fn render_form<T: PartialEq>(
    layout: &FormLayout<T>,
    style: &FormStyle,
    area: Rect,
    buf: &mut Buffer,
    state: &mut FormState<T>,
) {
    if state.fields.is_empty() {
        state.cursor_position = None;
        return;
    }
    match layout {
        FormLayout::Horizontal => render_horizontal(style, area, buf, state),
        FormLayout::Stacked => render_stacked(style, area, buf, state),
        FormLayout::Custom(custom) => render_custom(custom, style, area, buf, state),
    }
}

pub(crate) fn required_height<T: PartialEq>(
    layout: &FormLayout<T>,
    state: &FormState<T>,
    width: u16,
) -> u16 {
    match layout {
        FormLayout::Horizontal => required_height_horizontal(state, width),
        FormLayout::Stacked => required_height_stacked(state, width),
        FormLayout::Custom(custom) => required_height_custom(custom, state, width),
    }
}

pub(crate) fn count_lines(text: &str, max_width: u16) -> u16 {
    if text.is_empty() || max_width == 0 {
        return 0;
    }

    let mut lines = 1;
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = word.chars().count() as u16;

        if current_width == 0 {
            current_width = word_width;
        } else if current_width + 1 + word_width <= max_width {
            current_width += 1 + word_width;
        } else {
            lines += 1;
            current_width = word_width;
        }
    }

    lines
}

pub(crate) fn scroll_offset(heights: &[u16], viewport_height: u16, focus: usize) -> (usize, usize) {
    let total = heights.iter().sum();
    if viewport_height >= total {
        return (0, heights.len().saturating_sub(1));
    }
    let mut weights = Vec::new();
    let mut current = 0;
    for h in heights {
        weights.push((current, current + h));
        current += h;
    }
    let (_, right) = weights.get(focus).unwrap();

    let min = right.saturating_sub(viewport_height);

    let lowest = weights
        .iter()
        .position(|e| e.0 >= min)
        .unwrap_or(weights.len().saturating_sub(1));

    let max = weights[lowest].0 + viewport_height;
    let heighest = weights
        .iter()
        .skip(lowest)
        .position(|e| e.1 >= max)
        .map(|idx| idx + lowest)
        .unwrap_or(weights.len().saturating_sub(1));

    (lowest, heighest)
}

pub(crate) fn render_field<T>(
    area: Rect,
    buf: &mut Buffer,
    field: &mut Field<T>,
    value_style: Style,
    highlight_style: Style,
    placeholder_style: Style,
) -> Option<(u16, u16)> {
    match field.kind {
        FieldKind::SingleLine(ref mut single_line) => render_singleline(
            area,
            buf,
            single_line,
            value_style,
            highlight_style,
            placeholder_style,
        ),
        FieldKind::CheckBox(ref mut checkbox) => {
            render_checkbox(area, buf, checkbox, value_style, highlight_style)
        }
        FieldKind::Select(ref mut select) => {
            render_select(area, buf, select, value_style, highlight_style)
        }
        FieldKind::TextArea(ref mut text_area) => render_textarea(
            area,
            buf,
            text_area,
            value_style,
            highlight_style,
            placeholder_style,
        ),
    }
}

#[cfg(test)]
mod scroll_offset_tests {
    use super::*;

    #[test]
    fn focus_that_already_fits_stays_at_zero() {
        let heights = [2, 2, 6, 2];
        assert_eq!(scroll_offset(&heights, 9, 0), (0, 2));
        assert_eq!(scroll_offset(&heights, 9, 1), (0, 2));
    }

    #[test]
    fn minimal_scroll_reveals_focus_end_exactly() {
        let heights = [2, 2, 6, 2];
        assert_eq!(scroll_offset(&heights, 9, 2), (1, 3));
    }

    #[test]
    fn minimal_scroll_can_partially_hide_a_preceding_field() {
        let heights = [2, 2, 6, 2];
        assert_eq!(scroll_offset(&heights, 9, 3), (2, 3));
    }

    #[test]
    fn no_longer_anchors_when_focus_already_fits_from_zero() {
        let heights = [5, 5, 3, 3, 3];
        assert_eq!(scroll_offset(&heights, 10, 1), (0, 1));
    }

    #[test]
    fn minimal_scroll_can_leave_a_field_partially_visible_on_both_sides() {
        let heights = [5, 10, 5, 5];
        assert_eq!(scroll_offset(&heights, 10, 2), (2, 3));
    }

    #[test]
    fn boundary_field_with_zero_visible_rows_is_excluded() {
        let heights = [5, 5, 5];
        assert_eq!(scroll_offset(&heights, 10, 0), (0, 1));
    }

    #[test]
    fn field_taller_than_viewport_still_anchors_at_its_start() {
        let heights = [3, 3, 15];
        assert_eq!(scroll_offset(&heights, 10, 2), (2, 2));
    }

    #[test]
    fn single_field_taller_than_viewport_is_shown_alone() {
        let heights = [20];
        assert_eq!(scroll_offset(&heights, 10, 0), (0, 0));
    }

    #[test]
    fn focus_already_visible_from_zero_no_snap_penalty() {
        let heights = [2, 2, 6, 2];
        assert_eq!(scroll_offset(&heights, 9, 0), (0, 2));
    }

    #[test]
    fn bottom_can_still_be_cut_in_half() {
        let heights2 = [3, 3, 3, 3];
        let (_, highest2) = scroll_offset(&heights2, 8, 0);
        assert_eq!(highest2, 2);
    }
}
