use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crate::{
    Form, FormState,
    field::{Field, FieldKind},
    style::FieldState,
    widget::{check_box::render_checkbox, select::render_select, single_line::render_singleline},
};

impl<T: PartialEq> StatefulWidget for Form<T> {
    type State = FormState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Block::default().on_light_yellow().render(area, buf);
        let heights: Vec<_> = state.fields.iter().map(|f| f.options.height + 1).collect();
        let (from_field, to_field) = scroll_offset(&heights, area.height, state.focus);

        let constraints: Vec<_> = state.fields[from_field..=to_field]
            .iter()
            .map(|f| Constraint::Length(f.options.height + 1))
            .collect();
        let rows = Layout::vertical(constraints).split(area);
        let label_width = state.max_label_length() as u16 + 2;
        state.cursor_position = None;

        for (idx, field) in state.fields[from_field..=to_field].iter_mut().enumerate() {
            let has_focus = (idx + from_field) == state.focus;
            let field_state = field.options.to_field_state(has_focus);

            let [row, error] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(rows[idx]);

            let [left, right] =
                Layout::horizontal([Constraint::Length(label_width), Constraint::Fill(1)])
                    .areas(row);

            let label = Span::raw(field.label()).style(self.style.label.style_for(&field_state));
            label.render(left, buf);

            if let Some(position) = render_field(
                right,
                buf,
                field,
                self.style.value.style_for(&field_state),
                self.style.highlight.style_for(&field_state),
            ) && has_focus
            {
                state.cursor_position = Some(position);
            }

            if let Some(message) = field.error.as_ref() {
                let len = message.chars().count() as u16;
                let [_, right] =
                    Layout::horizontal([Constraint::Fill(1), Constraint::Length(len)]).areas(error);
                let error_message = Paragraph::new(message.as_str())
                    .style(self.style.error)
                    .alignment(HorizontalAlignment::Right);
                error_message.render(right, buf);
            }
        }
    }
}

pub fn render_field<T>(
    area: Rect,
    buf: &mut Buffer,
    field: &mut Field<T>,
    value_style: Style,
    highlight_style: Style,
) -> Option<(u16, u16)> {
    match field.kind {
        FieldKind::SingleLine(ref mut single_line) => {
            render_singleline(area, buf, single_line, value_style, highlight_style)
        }
        FieldKind::CheckBox(ref mut checkbox) => {
            render_checkbox(area, buf, checkbox, value_style, highlight_style)
        }
        FieldKind::Select(ref mut select) => {
            render_select(area, buf, select, value_style, highlight_style)
        }
    }
}

fn scroll_offset(heights: &[u16], viewport_height: u16, focus: usize) -> (usize, usize) {
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
    let (left, right) = weights.get(focus).unwrap();

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

#[cfg(test)]
mod tests {
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
    fn no_snap_needed_when_raw_offset_already_a_field_boundary() {
        let heights = [2, 2, 6, 2];
        let (lower, _) = scroll_offset(&heights, 9, 3);
    }

    #[test]
    fn focus_already_visible_from_zero_no_snap_penalty() {
        let heights = [2, 2, 6, 2];
        assert_eq!(scroll_offset(&heights, 9, 0), (0, 2));
    }

    #[test]
    fn bottom_can_still_be_cut_in_half() {
        let heights = [5, 10, 5, 5];
        let (_, highest) = scroll_offset(&heights, 10, 2);
        let heights2 = [3, 3, 3, 3];
        let (_, highest2) = scroll_offset(&heights2, 8, 0);
        assert_eq!(highest2, 2);
    }
}
