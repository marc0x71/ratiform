use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
    style::{Color, Style},
};
use ratiform::{
    Form,
    builder::FormBuilder,
    style::{FieldStyle, FormStyle},
    validators,
};

fn main() -> std::io::Result<()> {
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(1, "Nome")
            .placeholder("Insersci il nome".to_owned())
            .validator(validators::min_length(
                2,
                "Il nome deve avere una lunghezza di almeno 2 caratteri".to_owned(),
            ))
            .required("Campo obbligatorio".to_owned())
            .single_line(2, "Cognome")
            .placeholder("Insersci il cognome".to_owned())
            .validator(validators::min_length(
                2,
                "Il cognome deve avere una lunghezza di almeno 2 caratteri".to_owned(),
            ))
            .validator(validators::max_length(
                10,
                "Il cognome deve avere una lunghezza massima di 10 caratteri".to_owned(),
            ))
            .select(3, "Paese")
            .values_ref(&[("I", "Italia"), ("F", "Francia"), ("D", "Germania")])
            .selected(1)
            .height(5)
            .checkbox(4, "Accetto i termini")
            .checked(false)
            .optional()
            .single_line(5, "Password")
            .placeholder("Insersci la password".to_owned())
            .masked_with('•')
            .required("La password non può essere vuota".to_owned())
            .single_line(10, "Debug")
            .disabled()
            .build();

        loop {
            state.set_value(&4, "true");
            let f = format!("focus={:?}", state.focus_field());
            state.set_value(&10, &f);
            terminal.draw(|frame| {
                let [area, _] = Layout::vertical([Constraint::Length(19), Constraint::Fill(1)])
                    .areas(frame.area());
                frame.render_stateful_widget(Form::with_style(my_style()), area, &mut state);
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
                        let values: HashMap<i32, String> = state.values().collect();
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

fn my_style() -> FormStyle {
    let normal = Style::default().fg(Color::LightGreen);
    FormStyle::builder()
        .label(
            FieldStyle::builder()
                .normal(normal)
                .focused(normal.bold())
                .disabled(normal.crossed_out())
                .build(),
        )
        .value(
            FieldStyle::builder()
                .normal(normal)
                .focused(normal.bold())
                .disabled(normal.crossed_out())
                .build(),
        )
        .highlight(
            FieldStyle::builder()
                .normal(normal)
                .focused(normal.reversed())
                .disabled(normal.reversed().crossed_out())
                .build(),
        )
        .error(Style::default().bg(Color::Red).fg(Color::White).bold())
        .placeholder(normal.italic())
        .build()
}
