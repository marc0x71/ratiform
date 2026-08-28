#![allow(unused)]

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    macros::constraints,
    text::Span,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::{
    FormLayout, FormState,
    field::Field,
    layout::custom::{CustomLayout, Object, ObjectKind},
    render::{count_lines, render_field, scroll_offset},
    style::FormStyle,
};

pub(crate) fn render_custom<T: PartialEq>(
    layout: &CustomLayout<T>,
    style: &FormStyle,
    area: Rect,
    buf: &mut Buffer,
    state: &mut FormState<T>,
) {
    let heights = compute_heights(layout, &state.fields, area.width);

    let (from_row, to_row) = scroll_offset(&heights, area.height, state.focus);

    let constraints: Vec<_> = heights[from_row..=to_row]
        .iter()
        .map(|f| Constraint::Length(*f))
        .collect();
    let rows = Layout::vertical(constraints).split(area);

    for (idx, area) in rows[from_row..=to_row].iter().enumerate() {
        //
        let row = &layout.rows[idx];
        render_row(idx, row, style, state, *area, buf);
    }
}

fn render_row<T: PartialEq>(
    idx: usize,
    row: &[(Constraint, Option<Object<T>>)],
    style: &FormStyle,
    state: &mut FormState<T>,
    area: Rect,
    buf: &mut Buffer,
) {
    let cols = Layout::horizontal(row.iter().map(|(c, _)| *c).collect::<Vec<_>>()).split(area);
    for (idx, (_, object)) in row.iter().enumerate() {
        //
        if let Some(object) = object {
            render_object(object, style, state, cols[idx], buf);
        }
    }
}

fn render_object<T: PartialEq>(
    object: &Object<T>,
    style: &FormStyle,
    state: &mut FormState<T>,
    area: Rect,
    buf: &mut Buffer,
) {
    let Some(field_position) = find_field_position(&state.fields, &object.id) else {
        return;
    };
    let has_focus = state.fields[state.focus].id == object.id;
    let field = &mut state.fields[field_position];
    let field_state = field.options.to_field_state(has_focus);
    match object.kind {
        ObjectKind::Label => {
            let label = Paragraph::new(field.label())
                .style(style.label.style_for(&field_state))
                .wrap(Wrap { trim: true });
            label.render(area, buf);
        }
        ObjectKind::Value => {
            let a = render_field(
                area,
                buf,
                field,
                style.value.style_for(&field_state),
                style.highlight.style_for(&field_state),
                style.placeholder,
            );
        }
        ObjectKind::Error => {
            if let Some(message) = field.error.as_ref() {
                let len = message.chars().count() as u16;
                let [_, right] =
                    Layout::horizontal([Constraint::Fill(1), Constraint::Length(len)]).areas(area);
                let error_message = Span::raw(message.as_str()).style(style.error);
                error_message.render(right, buf);
            }
        }
    }
}

fn compute_heights<T: PartialEq>(
    layout: &CustomLayout<T>,
    fields: &[Field<T>],
    width: u16,
) -> Vec<u16> {
    layout
        .rows
        .iter()
        .map(|row| compute_row_height(row, fields, width))
        .collect()
}

fn compute_rows_heights<T: PartialEq>(
    layout: &CustomLayout<T>,
    fields: &[Field<T>],
    width: u16,
) -> u16 {
    layout
        .rows
        .iter()
        .map(|row| compute_row_height(row, fields, width))
        .sum()
}

fn compute_row_height<T: PartialEq>(
    row: &[(Constraint, Option<Object<T>>)],
    fields: &[Field<T>],
    width: u16,
) -> u16 {
    let constraints = Layout::horizontal(row.iter().map(|(c, _)| *c).collect::<Vec<_>>())
        .split(Rect::new(0, 0, width, 1));
    let mut max_height = 0;
    for (idx, (_, obj)) in row.iter().enumerate() {
        let h = match obj {
            Some(obj) => object_height(constraints[idx], fields, obj),
            None => 1,
        };
        max_height = max_height.max(h)
    }
    max_height
}

fn object_height<T: PartialEq>(area: Rect, fields: &[Field<T>], obj: &Object<T>) -> u16 {
    if let Some(field) = find_field(fields, &obj.id) {
        match obj.kind {
            ObjectKind::Label => count_lines(field.label(), area.width),
            ObjectKind::Value => field.options.height,
            ObjectKind::Error => 1,
        }
    } else {
        // id not in FormState.fields
        0
    }
}

fn find_field<'a, T: PartialEq>(fields: &'a [Field<T>], id: &T) -> Option<&'a Field<T>> {
    fields.iter().find(|f| f.id == *id)
}

fn find_field_position<T: PartialEq>(fields: &[Field<T>], id: &T) -> Option<usize> {
    fields.iter().position(|f| f.id == *id)
}

pub(crate) fn required_height_custom<T: PartialEq>(
    layout: &CustomLayout<T>,
    state: &FormState<T>,
    width: u16,
) -> u16 {
    compute_rows_heights(layout, &state.fields, width)
}

#[cfg(test)]
mod required_height_tests {
    use super::*;
    use crate::{
        FormState,
        field::{Field, FieldKind, FieldOptions},
        layout::custom::{CustomLayout, Object, ObjectKind},
        widget::single_line::SingleLineStatus,
    };
    use ratatui::layout::Constraint;

    fn make_field(id: i32, label: &str, height: u16) -> Field<i32> {
        Field {
            id,
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

    fn make_field_with_error(id: i32, label: &str, height: u16, error: Option<&str>) -> Field<i32> {
        Field {
            error: error.map(str::to_owned),
            ..make_field(id, label, height)
        }
    }

    fn lbl(id: i32) -> Option<Object<i32>> {
        Some(Object::new(ObjectKind::Label, id))
    }
    fn val(id: i32) -> Option<Object<i32>> {
        Some(Object::new(ObjectKind::Value, id))
    }
    fn err(id: i32) -> Option<Object<i32>> {
        Some(Object::new(ObjectKind::Error, id))
    }

    // ---------- casi base ----------

    #[test]
    fn an_empty_layout_requires_zero_height() {
        let custom = CustomLayout::<i32>::new(vec![]);
        let state = FormState::new(vec![], None);
        assert_eq!(required_height_custom(&custom, &state, 50), 0);
    }

    #[test]
    fn a_lone_value_cell_uses_the_field_configured_height() {
        let state = FormState::new(vec![make_field(1, "Name", 1)], None);
        let custom = CustomLayout::new(vec![vec![(Constraint::Fill(1), val(1))]]);
        assert_eq!(required_height_custom(&custom, &state, 50), 1);
    }

    // ---------- massimo tra le celle di una riga ----------

    #[test]
    fn value_height_wins_over_a_short_label() {
        let fields = vec![make_field(1, "Ok", 1), make_field(2, "Notes", 3)];
        let state = FormState::new(fields, None);
        let custom = CustomLayout::new(vec![vec![
            (Constraint::Length(10), lbl(1)),
            (Constraint::Fill(1), val(2)),
        ]]);
        // colonne su width=30: [10, 20]. "Ok" a 10 -> 1 riga. Value height=3. max=3.
        assert_eq!(required_height_custom(&custom, &state, 30), 3);
    }

    #[test]
    fn a_wrapped_label_can_require_more_height_than_the_value() {
        let fields = vec![
            make_field(1, "Nome cognome indirizzo", 1),
            make_field(2, "Zip", 1),
        ];
        let state = FormState::new(fields, None);
        let custom = CustomLayout::new(vec![vec![
            (Constraint::Length(10), lbl(1)),
            (Constraint::Fill(1), val(2)),
        ]]);
        // colonne su width=30: [10, 20]. "Nome cognome indirizzo" a 10 -> 3 righe
        // (Nome / cognome / indirizzo). Value height=1. max=3.
        assert_eq!(required_height_custom(&custom, &state, 30), 3);
    }

    // ---------- il wrap dipende dalla larghezza di COLONNA risolta, non dalla larghezza totale ----------

    #[test]
    fn the_label_wraps_according_to_its_resolved_column_width_not_the_total_width() {
        let state = FormState::new(vec![make_field(1, "Nome cognome indirizzo", 1)], None);

        // Length(10) da sola risolve esattamente a 10 -> 3 righe.
        let narrow = CustomLayout::new(vec![vec![(Constraint::Length(10), lbl(1))]]);
        assert_eq!(required_height_custom(&narrow, &state, 100), 3);

        // Stessa larghezza TOTALE (100), ma Fill(1) da sola prende tutti i 100 -> 1 riga.
        let wide = CustomLayout::new(vec![vec![(Constraint::Fill(1), lbl(1))]]);
        assert_eq!(required_height_custom(&wide, &state, 100), 1);
    }

    // ---------- somma su più righe (Email/Password del tuo example) ----------

    #[test]
    fn sums_the_heights_of_every_row() {
        let fields = vec![make_field(1, "Email", 1), make_field(2, "Password", 1)];
        let state = FormState::new(fields, None);
        let custom = CustomLayout::new(vec![
            vec![
                (Constraint::Length(15), lbl(1)),
                (Constraint::Fill(1), lbl(2)),
            ],
            vec![
                (Constraint::Length(15), val(1)),
                (Constraint::Fill(1), val(2)),
            ],
            vec![
                (Constraint::Length(15), err(1)),
                (Constraint::Fill(1), err(2)),
            ],
        ]);
        // ogni riga: max=1 (label corta, value height=1, error fisso a 1) -> 1+1+1=3.
        assert_eq!(required_height_custom(&custom, &state, 50), 3);
    }

    // ---------- None come spaziatore (punto 3) ----------

    #[test]
    fn a_row_made_entirely_of_none_cells_is_a_one_line_spacer() {
        let state = FormState::<i32>::new(vec![], None);
        let custom = CustomLayout::new(vec![vec![
            (Constraint::Fill(1), None),
            (Constraint::Length(10), None),
        ]]);
        assert_eq!(required_height_custom(&custom, &state, 30), 1);
    }

    #[test]
    fn a_none_cell_never_dominates_a_taller_sibling() {
        let state = FormState::new(vec![make_field(1, "Notes", 4)], None);
        let custom = CustomLayout::new(vec![vec![
            (Constraint::Length(10), None),
            (Constraint::Fill(1), val(1)),
        ]]);
        assert_eq!(required_height_custom(&custom, &state, 30), 4);
    }

    // ---------- Error: sempre 1 riga, mai wrap (punti 1/2) ----------

    #[test]
    fn error_cell_height_is_fixed_at_one_line_regardless_of_message() {
        let custom = CustomLayout::new(vec![vec![(Constraint::Length(15), err(1))]]);

        let with_error = FormState::new(
            vec![make_field_with_error(
                1,
                "Email",
                1,
                Some("Questo indirizzo email non è valido e supera abbondantemente una riga"),
            )],
            None,
        );
        let without_error = FormState::new(vec![make_field(1, "Email", 1)], None);

        assert_eq!(required_height_custom(&custom, &with_error, 50), 1);
        assert_eq!(required_height_custom(&custom, &without_error, 50), 1);
    }

    // ---------- id sconosciuto: fallback silenzioso, altezza 0 ----------

    #[test]
    fn a_cell_referencing_an_unknown_field_id_contributes_zero_height() {
        let state = FormState::<i32>::new(vec![], None);
        let custom = CustomLayout::new(vec![vec![(Constraint::Fill(1), val(999))]]);
        assert_eq!(required_height_custom(&custom, &state, 50), 0);
    }

    #[test]
    fn an_unknown_field_id_does_not_affect_the_height_of_its_row_siblings() {
        let state = FormState::new(vec![make_field(1, "Notes", 3)], None);
        let custom = CustomLayout::new(vec![vec![
            (Constraint::Fill(1), val(999)),  // id sconosciuto, contribuisce 0
            (Constraint::Length(10), val(1)), // height=3, vince lui
        ]]);
        assert_eq!(required_height_custom(&custom, &state, 30), 3);
    }

    #[test]
    fn unknown_id_rows_differ_from_none_spacer_rows() {
        let state = FormState::<i32>::new(vec![], None);

        let all_none = CustomLayout::new(vec![vec![(Constraint::Fill(1), None)]]);
        assert_eq!(required_height_custom(&all_none, &state, 30), 1); // spaziatore deliberato

        let all_unknown = CustomLayout::new(vec![vec![(Constraint::Fill(1), val(999))]]);
        assert_eq!(required_height_custom(&all_unknown, &state, 30), 0); // riferimento rotto
    }
}
