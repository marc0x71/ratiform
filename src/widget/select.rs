use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{List, ListState, StatefulWidget},
};

pub struct SelectStatus {
    pub(crate) label: String,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) list_state: ListState,
}

impl SelectStatus {
    fn up(&mut self) {
        self.list_state.select_previous();
    }

    fn down(&mut self) {
        self.list_state.select_next();
    }

    fn home(&mut self) {
        self.list_state.select_first();
    }

    fn end(&mut self) {
        self.list_state.select_last();
    }

    fn page_up(&mut self) {
        self.list_state.scroll_up_by(8);
    }

    fn page_down(&mut self) {
        self.list_state.scroll_down_by(8);
    }
}

pub(crate) fn handle_input_select(key_code: KeyCode, select: &mut SelectStatus) {
    match key_code {
        KeyCode::Up => select.up(),
        KeyCode::Down => select.down(),
        KeyCode::Home => select.home(),
        KeyCode::End => select.end(),
        KeyCode::PageUp => select.page_up(),
        KeyCode::PageDown => select.page_down(),
        _ => {}
    }
}

pub(crate) fn render_select(
    area: Rect,
    buf: &mut Buffer,
    select: &mut SelectStatus,
    has_focus: bool,
) -> Option<(u16, u16)> {
    let items: Vec<_> = select.values.iter().map(|(k, v)| v.as_str()).collect();

    let style = Style::default().fg(Color::Gray);

    let list = List::new(items)
        .style(style)
        .highlight_style(if has_focus {
            Modifier::REVERSED
        } else {
            Modifier::ITALIC
        })
        .highlight_symbol("> ");

    StatefulWidget::render(list, area, buf, &mut select.list_state);

    None
}
