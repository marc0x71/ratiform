use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
};
use ratiform::{Form, builder::FormBuilder, validators};

#[derive(Debug, Hash, Eq, PartialEq)]
enum ConnectionField {
    Host,
    Port,
    Protocol,
    Username,
    Password,
    SaveCredentials,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = FormBuilder::new()
        .single_line(ConnectionField::Host, "Host")
        .value("localhost")
        .required("Host is required".to_owned())
        // alphabet() rejects a character at the keystroke, before it
        // ever becomes part of the value -- parsable() still checks
        // the result is a valid u16.
        .single_line(ConnectionField::Port, "Port")
        .alphabet("0123456789")
        .value("5432")
        .validator(validators::parsable::<u16>(
            "Port must be a number between 0 and 65535".to_owned(),
        ))
        .required("Port is required".to_owned())
        .select(ConnectionField::Protocol, "Protocol")
        .values_ref(&[("tcp", "TCP"), ("ssl", "TCP + SSL")])
        .selected(0)
        .height(2)
        .single_line(ConnectionField::Username, "Username")
        .placeholder("postgres")
        .optional()
        .single_line(ConnectionField::Password, "Password")
        .masked()
        .optional()
        .checkbox(ConnectionField::SaveCredentials, "Save credentials")
        .checked(false)
        .optional()
        .build()?;

    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        loop {
            terminal.draw(|frame| {
                let [area, _] = Layout::vertical([Constraint::Length(16), Constraint::Fill(1)])
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
                        let values: HashMap<ConnectionField, String> = state.values().collect();
                        break Ok(values);
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })?;

    println!("Collected configuration: {result:?}");
    Ok(())
}
