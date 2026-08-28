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

pub(crate) fn render_horizontal<T: PartialEq>(
    style: &FormStyle,
    area: Rect,
    buf: &mut Buffer,
    state: &mut FormState<T>,
) {
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
        let (row, error) = if rows[idx].height >= 2 {
            // there is enough space for the error message
            let [row, error] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(rows[idx]);
            (row, Some(error))
        } else {
            // display only the widget
            (rows[idx], None)
        };

        let [left, right] =
            Layout::horizontal([Constraint::Length(label_width), Constraint::Fill(1)]).areas(row);

        let label = Paragraph::new(field.label())
            .style(style.label.style_for(&field_state))
            .wrap(Wrap { trim: true });
        label.render(left, buf);

        if let Some(position) = render_field(
            right,
            buf,
            field,
            style.value.style_for(&field_state),
            style.highlight.style_for(&field_state),
            style.placeholder,
        ) && has_focus
        {
            state.cursor_position = Some(position);
        }

        if let Some(message) = field.error.as_ref()
            && let Some(err_area) = error
        {
            let len = message.chars().count() as u16;
            let [_, right] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(len)]).areas(err_area);
            let error_message = Span::raw(message.as_str()).style(style.error);
            error_message.render(right, buf);
        }
    }
}

pub(crate) fn required_height_horizontal<T: PartialEq>(state: &FormState<T>, width: u16) -> u16 {
    let label_width =
        resolve_label_width(state.label_width, state.max_label_length() as u16, width);

    let heights: Vec<u16> = compute_heights(&state.fields, label_width);
    heights.iter().sum()
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

#[cfg(test)]
mod label_width_tests {
    use super::*;
    use crate::{
        field::{Field, FieldKind, FieldOptions},
        widget::single_line::SingleLineStatus,
    };

    pub(super) fn make_field(label: &str, height: u16) -> Field<i32> {
        Field {
            id: 1,
            kind: FieldKind::SingleLine(SingleLineStatus {
                label: label.to_owned(),
                value: String::new(),
                position: 0,
                masked_with: None,
                placeholder: None,
                alphabet: None,
            }),
            options: FieldOptions {
                required: None,
                disabled: false,
                readonly: false,
                height,
                validator: vec![],
                normalizer: None,
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

#[cfg(test)]
mod required_height_tests {
    use super::label_width_tests::make_field;

    use super::*;

    #[test]
    fn required_height_sums_the_heights_of_every_field() {
        let state = FormState::new(vec![make_field("A", 1), make_field("BB", 1)], None);

        assert_eq!(required_height_horizontal(&state, 30), 4);
    }

    #[test]
    fn required_height_reflects_an_explicit_label_width() {
        let wide = FormState::new(vec![make_field("Nome completo", 1)], None);
        assert_eq!(required_height_horizontal(&wide, 100), 2); // 1 (single line) + 1 (error row)

        let narrow = FormState::new(vec![make_field("Nome completo", 1)], Some(6));
        assert_eq!(required_height_horizontal(&narrow, 100), 3); // 2 (wrapped) + 1 (error row)
    }
}
