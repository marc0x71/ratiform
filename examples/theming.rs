use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event},
    layout::Constraint,
    style::{Color, Style},
};
use ratiform::{
    Form,
    builder::FormBuilder,
    style::{FieldStyle, FormStyle},
    validators,
};

#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    FirstName,
    LastName,
    Country,
    Terms,
    Password,
    Debug,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = FormBuilder::new()
        // A deliberately long label -- wraps onto a second line
        // instead of squeezing the value column. See the README's
        // "Label width" section.
        .single_line(
            FormField::FirstName,
            "First name of the user to add to the database",
        )
        .placeholder("Enter the first name of the user you want to add")
        .validator(validators::min_length(
            2,
            "First name must be at least 2 characters".to_owned(),
        ))
        .required("This field is required".to_owned())
        .single_line(FormField::LastName, "Last name")
        .placeholder("Enter the last name")
        .validator(validators::min_length(
            2,
            "Last name must be at least 2 characters".to_owned(),
        ))
        .validator(validators::max_length(
            10,
            "Last name is at most 10 characters".to_owned(),
        ))
        .select(FormField::Country, "Country")
        .values_ref(&[("IT", "Italy"), ("FR", "France"), ("DE", "Germany")])
        .selected(1)
        .height(5)
        .checkbox(FormField::Terms, "I accept the terms")
        .checked(false)
        .optional()
        .single_line(FormField::Password, "Password")
        .placeholder("Enter the password")
        .masked_with('•')
        .required("Password cannot be empty".to_owned())
        // A disabled field used purely to show live form state below --
        // not something you'd normally ship, just handy for a demo.
        .single_line(FormField::Debug, "Debug")
        .disabled()
        .label_width(25)
        .build()?;

    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        loop {
            let focus = format!("focus={:?}", state.focused_field());
            state.set_value(&FormField::Debug, &focus);

            terminal.draw(|frame| {
                let area = frame
                    .area()
                    .centered(Constraint::Length(80), Constraint::Length(30));
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

// A custom FormStyle: five areas (label, value, highlight, error,
// placeholder), each resolved for you based on the field's state. This
// one deliberately gives labels and values distinct colors, not just
// different weights of the same one -- closer to how most people
// actually theme a form in practice.
fn my_style() -> FormStyle {
    let label = Style::default().fg(Color::Cyan);
    let value = Style::default().fg(Color::White);

    FormStyle::builder()
        .label(
            FieldStyle::builder()
                .normal(label)
                .focused(label.bold())
                .disabled(label.crossed_out())
                .build(),
        )
        .value(
            FieldStyle::builder()
                .normal(value)
                .focused(value.bold())
                .disabled(Style::default().fg(Color::DarkGray).crossed_out())
                .build(),
        )
        .highlight(
            FieldStyle::builder()
                .normal(value)
                .focused(value.bg(Color::Blue))
                .disabled(
                    Style::default()
                        .fg(Color::DarkGray)
                        .bg(Color::Blue)
                        .crossed_out(),
                )
                .build(),
        )
        .error(Style::default().bg(Color::Red).fg(Color::White).bold())
        .placeholder(Style::default().fg(Color::DarkGray).italic())
        .build()
}
