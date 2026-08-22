use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
};
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
            .values_ref(&[("I", "Italia"), ("F", "Francia"), ("D", "Germania")])
            .selected(1)
            .height(5)
            .required()
            .checkbox("Accetto i termini")
            .checked(false)
            .optional()
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
                        break Ok(());
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })
}
