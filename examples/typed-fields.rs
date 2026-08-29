use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
};
use ratiform::{Form, builder::FormBuilder};

// The field identifier is a real Rust type, checked by the compiler --
// not a string you could typo. See the README's "Field identifiers"
// section for why this is the crate's main pitch.
#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    FirstName,
    LastName,
    Country,
    Terms,
}

fn main() -> std::io::Result<()> {
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(FormField::FirstName, "First name")
            .value("Mario")
            .validator(|v| {
                (v.len() > 2)
                    .then_some(())
                    .ok_or_else(|| "First name must be longer than 2 characters".to_owned())
            })
            .single_line(FormField::LastName, "Last name")
            .value("Rossi")
            .select(FormField::Country, "Country")
            .values_ref(&[("IT", "Italy"), ("FR", "France"), ("DE", "Germany")])
            // .selected(1)
            .no_selection()
            .height(5)
            // required by default; .optional() is what actually changes
            // behavior -- an unchecked box here doesn't block submission.
            .checkbox(FormField::Terms, "I accept the terms")
            .checked(false)
            .optional()
            .build().unwrap();

        loop {
            terminal.draw(|frame| {
                let [area, _] = Layout::vertical([Constraint::Length(19), Constraint::Fill(1)])
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
                        // The whole point: this map is keyed by `FormField`,
                        // not by string -- no risk of a typo'd key.
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
