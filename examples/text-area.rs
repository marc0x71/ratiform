use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
    style::{Color, Style},
};
use ratiform::{
    Form,
    builder::FormBuilder,
    style::{FormStyle, Parts, States, Widgets},
};

#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    Title,
    Body,
    Terms,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let placeholder_text = String::from(
        "First line\nSecond line\nLorem ipsum dolor sit amet, consectetur adipiscing elit. \
         Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    );

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
        .build()?;

    let result = ratatui::run(|terminal| -> std::io::Result<_> {
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
    let label = Style::default().fg(Color::Cyan);
    let value = Style::default().fg(Color::White);

    FormStyle::builder()
        // LABEL — readonly non impostato, resta Style::default() come nell'originale
        .add(Widgets::ANY, Parts::LABEL, States::NORMAL, label)
        .add(Widgets::ANY, Parts::LABEL, States::FOCUSED, label.bold())
        .add(
            Widgets::ANY,
            Parts::LABEL,
            States::DISABLED,
            label.crossed_out(),
        )
        // VALUE — TEXT|ITEM|MARKER in un solo .add(), come per il tema di default
        .add(
            Widgets::ANY,
            Parts::TEXT | Parts::ITEM | Parts::MARKER,
            States::NORMAL,
            value,
        )
        .add(
            Widgets::ANY,
            Parts::TEXT | Parts::ITEM | Parts::MARKER,
            States::FOCUSED,
            value.bold(),
        )
        .add(
            Widgets::ANY,
            Parts::TEXT | Parts::ITEM | Parts::MARKER,
            States::DISABLED,
            Style::default().fg(Color::DarkGray).crossed_out(),
        )
        // HIGHLIGHT — AREA|ACTIVE in un solo .add(), stessa logica del tema di default
        .add(
            Widgets::ANY,
            Parts::AREA | Parts::ACTIVE,
            States::NORMAL,
            value,
        )
        .add(
            Widgets::ANY,
            Parts::AREA | Parts::ACTIVE,
            States::FOCUSED,
            value.bg(Color::Blue),
        )
        .add(
            Widgets::ANY,
            Parts::AREA | Parts::ACTIVE,
            States::DISABLED,
            Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Blue)
                .crossed_out(),
        )
        // ERROR e PLACEHOLDER — invariati, nessuna variazione per stato
        .add(
            Widgets::ANY,
            Parts::ERROR,
            States::ANY,
            Style::default().bg(Color::Red).fg(Color::White).bold(),
        )
        .add(
            Widgets::SINGLE_LINE | Widgets::TEXT_AREA,
            Parts::PLACEHOLDER,
            States::ANY,
            Style::default().fg(Color::DarkGray).italic(),
        )
        .build()
}
