use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
};
use ratiform::{Form, builder::FormBuilder, validators};

#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    FirstName,
    LastName,
    ReferenceCode,
    Email,
}

fn main() -> std::io::Result<()> {
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(FormField::FirstName, "First name")
            .required("First name is required".to_owned())
            .single_line(FormField::LastName, "Last name")
            .required("Last name is required".to_owned())
            // normalizer() rewrites every keystroke into a canonical form,
            // before validation runs on it -- here, forced uppercase.
            .single_line(FormField::ReferenceCode, "Reference code")
            .placeholder("ABC-1234")
            .normalizer(|s| s.to_uppercase())
            .validator(validators::max_length(
                16,
                "Reference code is at most 16 characters".to_owned(),
            ))
            .required("Reference code is required".to_owned())
            // Same idea, the other direction: forced lowercase.
            .single_line(FormField::Email, "Email")
            .placeholder("mario.rossi@example.com")
            .normalizer(|s| s.to_lowercase())
            .validator(|value: &str| {
                let valid = value.split('@').count() == 2
                    && value
                        .split('@')
                        .next_back()
                        .is_some_and(|domain| domain.contains('.'));

                valid
                    .then_some(())
                    .ok_or_else(|| "Invalid email address".to_owned())
            })
            .required("Email is required".to_owned())
            .build().unwrap();

        loop {
            terminal.draw(|frame| {
                let [area, _] = Layout::vertical([Constraint::Length(9), Constraint::Fill(1)])
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
                        let values: HashMap<FormField, String> = state.values().collect();
                        break Ok(values);
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })?;

    println!("Collected data: {result:?}");
    Ok(())
}
