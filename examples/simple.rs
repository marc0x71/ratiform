use ratatui::crossterm::event::{self, Event, KeyCode};
use ratiform::{Form, builder::FormBuilder};

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| {
        let mut state = FormBuilder::new()
            .single_line("Nome")
            .value("Mario")
            .required()
            .single_line("Cognome")
            .value("Rossi")
            .required()
            .select("Paese")
            .values(&["Italia", "Francia", "Germania"])
            .required()
            .checkbox("Accetto i termini")
            .checked(true)
            .optional()
            .build();

        loop {
            terminal.draw(|frame| {
                frame.render_stateful_widget(Form::default(), frame.area(), &mut state);
                if let Some(position) = state.cursor_position() {
                    println!("{position:?}");
                    frame.set_cursor_position(position);
                }
            })?;

            if let Event::Key(key) = event::read()?
                && key.kind == event::KeyEventKind::Press
            {
                state.handle_input(key);
                match state.result() {
                    ratiform::FormResult::Submitted | ratiform::FormResult::Cancelled => {
                        break Ok(());
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })
}
