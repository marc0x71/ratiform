use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Span,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    FormState,
    field::Field,
    render::{count_lines, render_field, scroll_offset},
    style::FormStyle,
};

pub(crate) fn render_stacked<T: PartialEq>(
    style: FormStyle,
    area: Rect,
    buf: &mut Buffer,
    state: &mut FormState<T>,
) {
    let heights: Vec<u16> = compute_heights(&state.fields, area.width);
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

        let label_height = count_lines(field.label(), area.width);
        let value_height = field.options.height;
        let areas = compute_areas(label_height, value_height, 1, rows[idx]);

        // label
        if !areas.is_empty() {
            let label = Paragraph::new(field.label())
                .style(style.label.style_for(&field_state))
                .wrap(Wrap { trim: true });
            label.render(areas[0], buf);
        }
        // value
        if areas.len() > 1
            && let Some(position) = render_field(
                areas[1],
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
            && areas.len() > 2
        {
            let len = message.chars().count() as u16;
            let [_, right] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(len)]).areas(areas[2]);
            let error_message = Span::raw(message.as_str()).style(style.error);
            error_message.render(right, buf);
        }
    }
}

pub(crate) fn required_height_stacked<T: PartialEq>(state: &FormState<T>, width: u16) -> u16 {
    compute_heights(&state.fields, width).into_iter().sum()
}

fn compute_areas(
    label_height: u16,
    value_height: u16,
    error_height: u16,
    area: Rect,
) -> Rc<[Rect]> {
    let heights = [label_height, value_height, error_height];

    let mut available = area.height;
    let mut constraints = Vec::with_capacity(heights.len());

    for height in heights {
        if height > available {
            break;
        }

        constraints.push(Constraint::Length(height));
        available -= height;
    }

    Layout::vertical(constraints).split(area)
}

fn compute_heights<T: PartialEq>(fields: &[Field<T>], width: u16) -> Vec<u16> {
    let mut heights = Vec::new();
    for field in fields {
        let label_height = count_lines(field.label(), width);
        let value_height = field.options.height;
        heights.push(value_height + label_height + 1);
    }
    heights
}
