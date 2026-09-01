use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
};
use ratiform::{Form, builder::FormBuilder};

#[derive(Debug, Hash, Eq, PartialEq)]
enum ProjectField {
    Tags,
    Permissions,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = FormBuilder::new()
        .multi_select(ProjectField::Tags, "Tags")
        .values_ref(&[
            ("bug", "Bug"),
            ("feature", "Feature"),
            ("docs", "Documentation"),
            ("backend", "Backend"),
            ("frontend", "Frontend"),
        ])
        .height(5)
        .optional()
        .multi_select(ProjectField::Permissions, "Permissions")
        .values_ref(&[("read", "Read"), ("write", "Write"), ("admin", "Admin")])
        .optional()
        .selected(&[10]) // "read" selezionato all'avvio
        .height(3)
        .required("Select at least one permission".to_owned())
        .build()?;

    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        loop {
            terminal.draw(|frame| {
                let [area, _] = Layout::vertical([Constraint::Length(14), Constraint::Fill(1)])
                    .areas(frame.area());

                frame.render_stateful_widget(Form::default(), area, &mut state);

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
                        // value() resta la stringa CSV grezza (es. "bug,docs");
                        // multi_select(...) da' invece i singoli valori gia'
                        // separati, comodo per non riparsare la CSV a mano.
                        let tags: Vec<String> = state
                            .multi_select(&ProjectField::Tags)
                            .map(|r| r.selected_values().map(str::to_owned).collect())
                            .unwrap_or_default();

                        let values: HashMap<ProjectField, String> = state.values().collect();
                        break Ok((tags, values));
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })?;

    let (tags, values) = result;
    println!("Selected tags: {tags:?}");
    println!("Raw field values (CSV format): {values:?}");
    Ok(())
}
