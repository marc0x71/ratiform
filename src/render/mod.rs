mod widget;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Span,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crate::{Form, FormState, render::widget::render_field};

impl StatefulWidget for Form {
    type State = FormState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let constraints = vec![Constraint::Length(2); state.fields.len()];
        let rows = Layout::vertical(constraints).split(area);
        let width = state.max_label_length() as u16 + 2;
        state.cursor_position = None;

        for (idx, field) in state.fields.iter_mut().enumerate() {
            let [row, _] =
                Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(rows[idx]);

            let [left, right] =
                Layout::horizontal([Constraint::Length(width), Constraint::Fill(1)]).areas(row);

            let label = Span::raw(field.label());
            label.render(left, buf);

            if let Some(position) = render_field(right, buf, field, state.focus == idx) {
                state.cursor_position = Some(position);
            }
        }
    }
}
