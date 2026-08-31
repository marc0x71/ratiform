// Requires `FormState::set_visible` (not yet implemented as of this diff —
// this example is the visual target for it, not a finished demo).
//
// F1 cycles through the three layout kinds — Horizontal, Stacked, Custom —
// so you can check that a hidden field behaves correctly in all of them:
//   - Horizontal/Stacked: `Discount` owns its own rows, so hiding it
//     collapses that space cleanly, no gap left behind.
//   - Custom: the grid below keeps `Discount`'s cells in fixed positions
//     next to `Email`'s; hiding it leaves that space blank rather than
//     letting `Email` reflow into it (see the design discussion this
//     example came out of — recomposing the grid for a true reflow is the
//     app's job, not the library's).
//
// Space toggles the checkbox when it has focus; Tab/BackTab move focus.

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::Constraint,
    widgets::{Block, Borders, Padding},
};
use ratiform::{
    Form, FormLayout, builder::FormBuilder, custom_layout, layout::custom::CustomLayout,
};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Field {
    Email,
    Discount,
    ShowDiscount,
}

#[derive(Clone, Copy)]
enum LayoutKind {
    Horizontal,
    Stacked,
    Custom,
}

impl LayoutKind {
    fn next(self) -> Self {
        match self {
            LayoutKind::Horizontal => LayoutKind::Stacked,
            LayoutKind::Stacked => LayoutKind::Custom,
            LayoutKind::Custom => LayoutKind::Horizontal,
        }
    }

    fn label(self) -> &'static str {
        match self {
            LayoutKind::Horizontal => "Horizontal",
            LayoutKind::Stacked => "Stacked",
            LayoutKind::Custom => "Custom",
        }
    }
}

fn custom_grid() -> CustomLayout<Field> {
    // Email | Discount, side by side — Discount's cells stay in this fixed
    // position whether the field is hidden or not.
    custom_layout! {
        row [
            (Constraint::Fill(1), Label(Field::Email)),
            (Constraint::Fill(1), Label(Field::Discount)),
        ],
        row [
            (Constraint::Fill(1), Value(Field::Email)),
            (Constraint::Fill(1), Value(Field::Discount)),
        ],
        row [
            (Constraint::Fill(1), Error(Field::Email)),
            (Constraint::Fill(1), Error(Field::Discount)),
        ],
        row [
            (Constraint::Fill(1), Label(Field::ShowDiscount)),
            (Constraint::Length(4), Value(Field::ShowDiscount)),
        ],
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = FormBuilder::new()
        .single_line(Field::Email, "Email")
        .single_line(Field::Discount, "Discount code")
        .hide()
        .checkbox(Field::ShowDiscount, "Have a discount code?")
        .build()?;

    let mut layout_kind = LayoutKind::Horizontal;

    ratatui::run(|terminal| -> std::io::Result<_> {
        loop {
            let show_discount = state.value_as::<bool>(&Field::ShowDiscount) == Some(Ok(true));
            state.set_visible(&Field::Discount, show_discount);

            let form_layout = match layout_kind {
                LayoutKind::Horizontal => FormLayout::Horizontal,
                LayoutKind::Stacked => FormLayout::Stacked,
                LayoutKind::Custom => FormLayout::Custom(custom_grid()),
            };
            let form = Form::default().with_layout(form_layout);

            terminal.draw(|frame| {
                let area = frame
                    .area()
                    .centered(Constraint::Length(60), Constraint::Length(14));
                let title = format!(" Form — F1: {} ", layout_kind.label());
                let block = Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .padding(Padding::uniform(1));
                let inner = block.inner(area);

                frame.render_widget(block, area);
                frame.render_stateful_widget(&form, inner, &mut state);

                if let Some(position) = state.cursor_position() {
                    frame.set_cursor_position(position);
                }
            })?;

            if let Event::Key(key) = event::read()?
                && key.kind == event::KeyEventKind::Press
            {
                if key.code == KeyCode::F(1) {
                    layout_kind = layout_kind.next();
                    continue;
                }

                state.handle_input(key);

                match state.result() {
                    ratiform::FormResult::Submitted | ratiform::FormResult::Cancelled => {
                        break Ok(());
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })?;

    Ok(())
}
