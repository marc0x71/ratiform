use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::{StatefulWidget, Widget},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, List, Paragraph},
};

use crate::field::{CheckBox, Field, FieldKind, Select, SingleLine};

pub fn render_field(
    area: Rect,
    buf: &mut Buffer,
    field: &mut Field,
    has_focus: bool,
) -> Option<(u16, u16)> {
    match field.kind {
        FieldKind::SingleLine(ref mut single_line) => {
            render_singleline(area, buf, single_line, has_focus)
        }
        FieldKind::CheckBox(ref mut checkbox) => render_checkbox(area, buf, checkbox, has_focus),
        FieldKind::Select(ref mut select) => render_select(area, buf, select, has_focus),
    }
}

fn render_singleline(
    area: Rect,
    buf: &mut Buffer,
    singleline: &mut SingleLine,
    has_focus: bool,
) -> Option<(u16, u16)> {
    let style = if has_focus {
        Style::default().fg(Color::Black).bg(Color::Gray)
    } else {
        Style::default().fg(Color::Gray)
    };

    let value = Paragraph::new(singleline.value.as_str()).block(Block::default().style(style));

    value.render(area, buf);

    if has_focus {
        Some((area.x + singleline.position, area.y))
    } else {
        None
    }
}

fn render_checkbox(
    area: Rect,
    buf: &mut Buffer,
    checkbox: &mut CheckBox,
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

fn render_select(
    area: Rect,
    buf: &mut Buffer,
    select: &mut Select,
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
