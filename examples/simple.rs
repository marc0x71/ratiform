use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
};
use ratiform::{Form, builder::FormBuilder};

fn main() -> std::io::Result<()> {
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(1, "Nome")
            .value("Mario")
            .validator(Box::new(|v| {
                (v.len() > 2)
                    .then_some(())
                    .ok_or_else(|| "Il nome deve avere una lunghezza maggiore di 2".to_owned())
            }))
            .required()
            .single_line(2, "Cognome")
            .value("Rossi")
            .required()
            .select(3, "Paese")
            .values_ref(&[("I", "Italia"), ("F", "Francia"), ("D", "Germania")])
            .selected(1)
            .height(5)
            .required()
            .checkbox(4, "Accetto i termini")
            .checked(false)
            .optional()
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
