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
        let label_width = resolve_label_width(
            state.label_width,
            state.max_label_length() as u16,
            area.width,
        );

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

fn resolve_label_width(configured: Option<u16>, computed_max: u16, area_width: u16) -> u16 {
    match configured {
        Some(width) => width + 1,
        None => (computed_max + 1).min(area_width / 3),
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

#[cfg(test)]
mod label_width_tests {
    use super::*;
    use crate::{
        field::{Field, FieldOptions},
        widget::single_line::SingleLineStatus,
    };

    fn make_field(label: &str, height: u16) -> Field<i32> {
        Field {
            id: 1,
            kind: FieldKind::SingleLine(SingleLineStatus {
                label: label.to_owned(),
                value: String::new(),
                position: 0,
                masked_with: None,
                placeholder: None,
            }),
            options: FieldOptions {
                required: None,
                disabled: false,
                readonly: false,
                height,
                validator: vec![],
            },
            error: None,
            initial_value: String::new(),
        }
    }

    // ---------- count_lines ----------

    #[test]
    fn wraps_at_the_word_boundary_when_it_would_exceed_the_width() {
        assert_eq!(count_lines("Nome cognome indirizzo", 15), 2);
    }

    #[test]
    fn a_single_word_longer_than_the_width_is_not_split() {
        assert_eq!(count_lines("Supercalifragilistichespiralidoso", 10), 1);
    }

    #[test]
    fn empty_text_returns_zero_lines() {
        assert_eq!(count_lines("", 10), 0);
    }

    #[test]
    fn zero_width_returns_zero_lines() {
        assert_eq!(count_lines("Nome cognome", 0), 0);
    }

    #[test]
    fn unicode_characters_are_counted_not_bytes() {
        assert_eq!(count_lines("Città natale", 12), 1);
    }

    // ---------- compute_heights ----------

    #[test]
    fn the_configured_height_wins_when_taller_than_the_wrapped_label() {
        let field = make_field("Paese", 5);
        let heights = compute_heights(std::slice::from_ref(&field), 20);
        assert_eq!(heights[0], 6);
    }

    #[test]
    fn the_wrapped_label_wins_when_taller_than_the_configured_height() {
        let field = make_field("Nome cognome indirizzo", 1);
        let heights = compute_heights(std::slice::from_ref(&field), 15);
        assert_eq!(heights[0], 3);
    }

    // ---------- resolve_label_width ----------

    #[test]
    fn explicit_label_width_is_not_capped_even_when_wider_than_a_third_of_the_area() {
        assert_eq!(resolve_label_width(Some(200), 5, 80), 201);
    }

    #[test]
    fn auto_computed_label_width_is_capped_to_a_third_of_the_area() {
        assert_eq!(resolve_label_width(None, 50, 60), 20);
    }

    #[test]
    fn explicit_and_auto_computed_paths_add_the_same_padding() {
        let explicit = resolve_label_width(Some(10), 0, 100);
        let auto = resolve_label_width(None, 10, 100);
        assert_eq!(explicit, auto);
    }
}
