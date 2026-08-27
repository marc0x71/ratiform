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

#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    Title,
    Body,
    Terms,
}

fn main() -> std::io::Result<()> {
    let placeholder_text = String::from(
        "First line\nSecond line\nLorem ipsum dolor sit amet, consectetur adipiscing elit. \
         Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    );

    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(FormField::Title, "Title")
            .required("Title is required".to_owned())
            // A multi-line field: wraps long lines, scrolls vertically,
            // Ctrl+Enter submits since Enter itself inserts a newline.
            .text_area(FormField::Body, "Body")
            .placeholder("Write the article body here...")
            .value(placeholder_text)
            .height(5)
            .checkbox(FormField::Terms, "I accept the terms")
            .checked(false)
            .optional()
            .build();

        loop {
            terminal.draw(|frame| {
                let [area, _] = Layout::vertical([Constraint::Length(19), Constraint::Fill(1)])
                    .areas(frame.area());
                frame.render_stateful_widget(
                    Form::default().with_style(my_style()),
                    area,
                    &mut state,
                );
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
