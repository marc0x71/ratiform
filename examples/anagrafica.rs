use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
};
use ratiform::{Form, builder::FormBuilder, validators};

#[derive(Debug, Hash, Eq, PartialEq)]
enum AnagraficaField {
    Nome,
    Cognome,
    CodiceFiscale,
    Email,
}

fn main() -> std::io::Result<()> {
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(AnagraficaField::Nome, "Nome")
            .required("Il nome è obbligatorio".to_owned())
            .single_line(AnagraficaField::Cognome, "Cognome")
            .required("Il cognome è obbligatorio".to_owned())
            .single_line(AnagraficaField::CodiceFiscale, "Codice fiscale")
            .placeholder("RSSMRA80A01H501U")
            .validator(validators::max_length(
                16,
                "Il codice fiscale ha al massimo 16 caratteri".to_owned(),
            ))
            .required("Il codice fiscale è obbligatorio".to_owned())
            .single_line(AnagraficaField::Email, "Email")
            .placeholder("mario.rossi@esempio.it")
            .validator(|value: &str| {
                let valid = value.split('@').count() == 2
                    && value
                        .split('@')
                        .next_back()
                        .is_some_and(|domain| domain.contains('.'));

                valid
                    .then_some(())
                    .ok_or_else(|| "Indirizzo email non valido".to_owned())
            })
            .required("L'email è obbligatoria".to_owned())
            .build();

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
                        let values: HashMap<AnagraficaField, String> = state.values().collect();
                        break Ok(values);
                    }
                    ratiform::FormResult::Working => {
                        // Codice fiscale: forzato in maiuscolo
                        if let Some(value) = state.value(&AnagraficaField::CodiceFiscale) {
                            let upper = value.to_uppercase();
                            if upper != value {
                                state.set_value(&AnagraficaField::CodiceFiscale, &upper);
                            }
                        }

                        // Email: forzata tutti in minuscolo
                        if let Some(value) = state.value(&AnagraficaField::Email) {
                            let lower = value.to_lowercase();
                            if lower != value {
                                state.set_value(&AnagraficaField::Email, &lower);
                            }
                        }
                    }
                }
            }
        }
    })?;

    println!("Dati raccolti: {result:?}");
    Ok(())
}
