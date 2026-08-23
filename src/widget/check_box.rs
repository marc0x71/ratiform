use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Widget,
};

pub struct CheckBoxStatus {
    pub(crate) label: String,
    pub(crate) checked: bool,
}

impl CheckBoxStatus {
    fn toggle(&mut self) {
        self.checked = !self.checked
    }
}

pub(crate) fn handle_input_checkbox(key_code: KeyCode, check_box: &mut CheckBoxStatus) {
    if let KeyCode::Char(' ') = key_code {
        check_box.toggle();
    }
}

pub(crate) fn render_checkbox(
    area: Rect,
    buf: &mut Buffer,
    checkbox: &mut CheckBoxStatus,
    has_focus: bool,
) -> Option<(u16, u16)> {
    let flag = if checkbox.checked { "[✓]" } else { "[ ]" };

    let style = if has_focus {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    let value = Span::raw(flag).style(style);

    value.render(area, buf);

    None
}
