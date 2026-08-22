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
        let heights: Vec<_> = state.fields.iter().map(|f| f.options.height + 1).collect();

        let constraints: Vec<_> = state
            .fields
            .iter()
            .map(|f| Constraint::Length(f.options.height + 1))
            .collect();
        let rows = Layout::vertical(constraints).split(area);
        let (from_field, to_field) = scroll_offset(&heights, area.height, state.focus);
        let label_width = state.max_label_length() as u16 + 2;
        state.cursor_position = None;

        for (idx, field) in state.fields.iter_mut().enumerate() {
            let [row, _] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(rows[idx]);

            let [left, right] =
                Layout::horizontal([Constraint::Length(label_width), Constraint::Fill(1)])
                    .areas(row);

            let label = Span::raw(field.label());
            label.render(left, buf);

            if let Some(position) = render_field(right, buf, field, state.focus == idx) {
                state.cursor_position = Some(position);
            }
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

    let lowest = weights
        .iter()
        .position(|e| e.1 > *left)
        .unwrap_or(weights.len().saturating_sub(1));

    let max = *left + viewport_height;
    let heightest = weights
        .iter()
        .position(|e| e.1 > max)
        .unwrap_or(weights.len().saturating_sub(1));

    (lowest, heightest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_content_fits_shows_everything_regardless_of_focus() {
        // totale = 9, viewport = 10: nessuno scroll necessario, qualunque sia il focus
        let heights = [3, 3, 3];
        assert_eq!(scroll_offset(&heights, 10, 0), (0, 2));
        assert_eq!(scroll_offset(&heights, 10, 1), (0, 2));
        assert_eq!(scroll_offset(&heights, 10, 2), (0, 2));
    }

    #[test]
    fn exact_fit_shows_everything() {
        // totale == viewport esattamente: deve comunque scattare lo shortcut
        let heights = [5, 5];
        assert_eq!(scroll_offset(&heights, 10, 0), (0, 1));
        assert_eq!(scroll_offset(&heights, 10, 1), (0, 1));
    }

    #[test]
    fn empty_heights_returns_degenerate_range() {
        // nessun campo: comportamento "di default" documentato, non necessariamente
        // significativo a valle (il chiamante deve gestire heights.len() == 0 a parte)
        let heights: [u16; 0] = [];
        assert_eq!(scroll_offset(&heights, 10, 0), (0, 0));
    }

    #[test]
    fn window_starts_exactly_at_focus_field() {
        // field0:[0,5) field1:[5,15) field2:[15,20) <- focus
        // il contenuto non entra tutto (25 > 10): la finestra parte
        // esattamente dall'inizio del campo con focus, mai prima
        let heights = [5, 10, 5, 5];
        assert_eq!(scroll_offset(&heights, 10, 2), (2, 3));
    }

    #[test]
    fn window_extends_forward_while_space_allows() {
        // focus=1 (righe [5,10)); la finestra [5,15) copre anche field2 [10,13)
        // per intero e field3 [13,16) solo parzialmente; field4 resta fuori
        let heights = [5, 5, 3, 3, 3];
        assert_eq!(scroll_offset(&heights, 10, 1), (1, 3));
    }

    #[test]
    fn single_field_taller_than_viewport_is_shown_alone() {
        let heights = [20];
        assert_eq!(scroll_offset(&heights, 10, 0), (0, 0));
    }

    #[test]
    fn first_field_taller_than_viewport_is_shown_alone() {
        let heights = [15, 3, 3];
        assert_eq!(scroll_offset(&heights, 10, 0), (0, 0));
    }

    #[test]
    fn last_field_taller_than_viewport_is_shown_alone() {
        let heights = [3, 3, 15];
        assert_eq!(scroll_offset(&heights, 10, 2), (2, 2));
    }

    #[test]
    fn highest_falls_back_to_last_field_when_window_extends_past_end() {
        // focus sull'ultimo campo: max (left + viewport) supera il totale,
        // nessun weights.1 lo supera -> scatta il fallback unwrap_or
        let heights = [3, 3, 3, 3]; // totale 12, viewport 10
        assert_eq!(scroll_offset(&heights, 10, 3), (3, 3));
    }

    #[test]
    fn field_ending_exactly_at_viewport_boundary() {
        // field0:[0,5) field1:[5,10) field2:[10,15)
        // dopo field0+field1 la somma è ESATTAMENTE 10 (== viewport):
        // field2 inizia proprio dove finisce la viewport, quindi a schermo
        // non se ne vedrebbe nemmeno una riga. Con l'implementazione attuale
        // viene comunque incluso nel range.
        let heights = [5, 5, 5];
        assert_eq!(scroll_offset(&heights, 10, 0), (0, 2));
    }

    #[test]
    #[should_panic]
    fn focus_out_of_bounds_panics() {
        let heights = [3, 3];
        scroll_offset(&heights, 1, 5); // total(6) > viewport(1): niente shortcut, focus fuori range
    }
}
