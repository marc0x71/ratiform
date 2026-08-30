use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    text::Line,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    FormState,
    field::Field,
    render::{count_lines, render_field, scroll_offset},
    style::FormStyle,
};

pub(crate) fn render_stacked<T: PartialEq>(
    style: &FormStyle,
    area: Rect,
    buf: &mut Buffer,
    state: &mut FormState<T>,
) {
    let heights: Vec<u16> = compute_heights(&state.fields, area.width);
    let focus_block = state.focus * 3 + 2;
    let (from_row, to_row) = scroll_offset(&heights, area.height, focus_block);

    let constraints: Vec<_> = heights[from_row..=to_row]
        .iter()
        .map(|f| Constraint::Length(*f))
        .collect();
    let rows = Layout::vertical(constraints).split(area);

    state.cursor_position = None;

    for (row_idx, row) in rows.iter().enumerate() {
        let field_index = (from_row + row_idx) / 3;
        let field = state.fields.get_mut(field_index).unwrap();
        let has_focus = field_index == state.focus;
        let field_state = field.options.to_field_state(has_focus);

        let area = *row;

        let element = (from_row + row_idx) % 3;
        // label
        if element == 0 {
            let label = Paragraph::new(field.label())
                .style(style.label.style_for(&field_state))
                .wrap(Wrap { trim: true });
            label.render(area, buf);
        }
        // value
        if element == 1
            && let Some(position) = render_field(
                area,
                buf,
                field,
                style.value.style_for(&field_state),
                style.highlight.style_for(&field_state),
                style.placeholder,
            )
            && has_focus
        {
            state.cursor_position = Some(position);
        }
        // error
        if let Some(message) = field.error.as_ref()
            && element == 2
        {
            let error_message = Paragraph::new(Line::styled(message.as_str(), style.error))
                .alignment(Alignment::Right)
                .wrap(Wrap { trim: true });
            error_message.render(area, buf);
        }
    }
}

pub(crate) fn required_height_stacked<T: PartialEq>(state: &FormState<T>, width: u16) -> u16 {
    compute_heights(&state.fields, width).into_iter().sum()
}

fn compute_heights<T: PartialEq>(fields: &[Field<T>], width: u16) -> Vec<u16> {
    let mut heights = Vec::new();
    for field in fields {
        let label_height = count_lines(field.label(), width);
        let value_height = field.options.height;
        let error_height = field
            .error
            .as_ref()
            .map(|msg| count_lines(msg.as_str(), width))
            .unwrap_or(1);
        heights.push(label_height);
        heights.push(value_height);
        heights.push(error_height);
    }
    heights
}
