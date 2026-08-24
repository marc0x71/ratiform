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

fn main() -> std::io::Result<()> {
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(ConnectionField::Host, "Host")
            .value("localhost")
            .required("L'host è obbligatorio".to_owned())
            .single_line(ConnectionField::Port, "Porta")
            .value("5432")
            .validator(validators::parsable::<u16>(
                "La porta deve essere un numero tra 0 e 65535".to_owned(),
            ))
            .required("La porta è obbligatoria".to_owned())
            .select(ConnectionField::Protocol, "Protocollo")
            .values_ref(&[("tcp", "TCP"), ("ssl", "TCP + SSL")])
            .selected(0)
            .height(2)
            .single_line(ConnectionField::Username, "Utente")
            .placeholder("postgres")
            .optional()
            .single_line(ConnectionField::Password, "Password")
            .masked()
            .optional()
            .checkbox(ConnectionField::SaveCredentials, "Salva le credenziali")
            .checked(false)
            .optional()
            .build();

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

    println!("Configurazione raccolta: {result:?}");
    Ok(())
}
