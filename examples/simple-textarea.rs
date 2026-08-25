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
};

fn main() -> std::io::Result<()> {
    let testo = String::from(
        "prima riga\nseconda riga\nLorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laborisnisi ut aliquip ex ea commodo consequat.",
    );
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(1, "Titolo")
            .required("Campo obbligatorio".to_owned())
            .text_area(2, "Articolo")
            .value(testo)
            .height(5)
            .checkbox(4, "Accetto i termini")
            .checked(false)
            .build();

        loop {
            state.set_value(&4, "true");
            let f = format!("focus={:?}", state.focused_field());
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
