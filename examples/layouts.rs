use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::Constraint,
    widgets::{Block, Padding},
};
use ratiform::{Form, builder::FormBuilder, layout::FormLayout};

#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    FirstName,
    LastName,
    Bio,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = FormBuilder::new()
        .single_line(FormField::FirstName, "First name")
        .required("First name is required".to_owned())
        .single_line(FormField::LastName, "Last name")
        .required("Last name is required".to_owned())
        .text_area(FormField::Bio, "Short bio")
        .placeholder("A couple of lines about yourself")
        .height(3)
        .optional()
        .build()?;

    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        loop {
            terminal.draw(|frame| {
                let area = frame
                    .area()
                    .centered(Constraint::Length(60), Constraint::Length(12));

                let block = Block::bordered()
                    .title(" Form ")
                    .padding(Padding::uniform(1));

                let inner = block.inner(area);
                frame.render_widget(block, area);

                // The "natural" use of FormLayout: recomputed from the
                // available width every frame, not chosen once and stored
                // -- resize the terminal and this switches on its own.
                let layout = if inner.width < 56 {
                    FormLayout::Stacked
                } else {
                    FormLayout::Horizontal
                };

                frame.render_stateful_widget(
                    Form::default().with_layout(layout),
                    inner,
                    &mut state,
                );
                if let Some(position) = state.cursor_position() {
                    frame.set_cursor_position(position);
                }
            })?;

            if let Event::Key(key) = event::read()?
                && key.kind == event::KeyEventKind::Press
            {
                state.handle_input(key);
                match state.result() {
                    ratiform::FormResult::Submitted | ratiform::FormResult::Cancelled => {
                        let values: HashMap<FormField, String> = state.values().collect();
                        break Ok(values);
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })?;
    println!("got = {result:?}");
    Ok(())
}
