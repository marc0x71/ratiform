use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::Constraint,
    widgets::{Block, Borders, Padding},
};
use ratiform::{Form, builder::FormBuilder, validators};

#[derive(Debug, Hash, Eq, PartialEq)]
enum LoginField {
    Username,
    Password,
    RememberMe,
}

fn main() -> std::io::Result<()> {
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(LoginField::Username, "Username")
            .placeholder("mario.rossi")
            .validator(validators::min_length(
                3,
                "L'username deve avere almeno 3 caratteri".to_owned(),
            ))
            .required("L'username è obbligatorio".to_owned())
            .single_line(LoginField::Password, "Password")
            .masked()
            .validator(validators::min_length(
                8,
                "La password deve avere almeno 8 caratteri".to_owned(),
            ))
            .validator(|value: &str| {
                let has_uppercase = value.chars().any(|c| c.is_uppercase());
                let has_lowercase = value.chars().any(|c| c.is_lowercase());
                let has_digit = value.chars().any(|c| c.is_ascii_digit());
                let has_special = value
                    .chars()
                    .any(|c| !c.is_alphanumeric() && !c.is_whitespace());

                (has_uppercase && has_lowercase && has_digit && has_special)
                    .then_some(())
                    .ok_or_else(|| {
                        "La password deve contenere almeno una maiuscola, una minuscola, \
                         un numero e un carattere speciale"
                            .to_owned()
                    })
            })
            .required("La password è obbligatoria".to_owned())
            .checkbox(LoginField::RememberMe, "Ricordami")
            .checked(false)
            .optional()
            .build();

        loop {
            terminal.draw(|frame| {
                let area = frame
                    .area()
                    .centered(Constraint::Length(50), Constraint::Length(10));
                let block = Block::default()
                    .title(" Login ")
                    .borders(Borders::ALL)
                    .padding(Padding::uniform(1));
                let inner = block.inner(area);

                frame.render_widget(block, area);
                frame.render_stateful_widget(Form::default(), inner, &mut state);

                if let Some(position) = state.cursor_position() {
                    frame.set_cursor_position(position);
                }
            })?;

            if let Event::Key(key) = event::read()?
                && key.kind == event::KeyEventKind::Press
            {
                state.handle_input(key);

                match state.result() {
                    ratiform::FormResult::Submitted => {
                        let values: HashMap<LoginField, String> = state.values().collect();
                        break Ok(Some(values));
                    }
                    ratiform::FormResult::Cancelled => {
                        break Ok(None);
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })?;

    match result {
        Some(values) => {
            let username = values
                .get(&LoginField::Username)
                .cloned()
                .unwrap_or_default();
            let remember = values
                .get(&LoginField::RememberMe)
                .map(|v| v == "true")
                .unwrap_or(false);

            // La password è disponibile in values.get(&LoginField::Password),
            // ma non la stampiamo di proposito: è solo un esempio.
            println!("Login effettuato: {username} (ricordami: {remember})");
        }
        None => println!("Accesso annullato."),
    }

    Ok(())
}
