use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{
    Form, FormState,
    field::{Field, FieldKind},
    widget::{
        check_box::render_checkbox, select::render_select, single_line::render_singleline,
        text_area::render_textarea,
    },
};

impl<T: PartialEq> StatefulWidget for Form<T> {
    type State = FormState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let label_width = if let Some(max_width) = state.label_width {
            max_width + 1
        } else {
            let width = state.max_label_length() as u16 + 2;
            width.min(area.width / 3)
        };

        let heights: Vec<u16> = compute_heights(&state.fields, label_width);

        let (from_field, to_field) = scroll_offset(&heights, area.height, state.focus);

        let constraints: Vec<_> = heights[from_field..=to_field]
            .iter()
            .map(|f| Constraint::Length(*f))
            .collect();
        let rows = Layout::vertical(constraints).split(area);

        state.cursor_position = None;

        for (idx, field) in state.fields[from_field..=to_field].iter_mut().enumerate() {
            let has_focus = (idx + from_field) == state.focus;
            let field_state = field.options.to_field_state(has_focus);

            let [row, error] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(rows[idx]);

            let [left, right] =
                Layout::horizontal([Constraint::Length(label_width), Constraint::Fill(1)])
                    .areas(row);

            let label = Paragraph::new(field.label())
                .style(self.style.label.style_for(&field_state))
                .wrap(Wrap { trim: true });
            label.render(left, buf);

            if let Some(position) = render_field(
                right,
                buf,
                field,
                self.style.value.style_for(&field_state),
                self.style.highlight.style_for(&field_state),
                self.style.placeholder,
            ) && has_focus
            {
                state.cursor_position = Some(position);
            }

            if let Some(message) = field.error.as_ref() {
                let len = message.chars().count() as u16;
                let [_, right] =
                    Layout::horizontal([Constraint::Fill(1), Constraint::Length(len)]).areas(error);
                let error_message = Span::raw(message.as_str()).style(self.style.error);
                error_message.render(right, buf);
            }
        }
    }
}

fn compute_heights<T: PartialEq>(fields: &[Field<T>], width: u16) -> Vec<u16> {
    let mut heights = Vec::new();
    for field in fields {
        let height = field.options.height.max(count_lines(field.label(), width)) + 1;
        heights.push(height);
    }
    heights
}

fn count_lines(text: &str, max_width: u16) -> u16 {
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
pub fn render_field<T>(
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
