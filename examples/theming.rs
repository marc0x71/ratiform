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
        .values(countries())
        .horizontal()
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

fn countries() -> Vec<(String, String)> {
    vec![
        ("A".to_string(), "Austria".to_string()),
        ("AL".to_string(), "Albania".to_string()),
        ("AND".to_string(), "Andorra".to_string()),
        ("AM".to_string(), "Armenia".to_string()),
        ("AZ".to_string(), "Azerbaijan".to_string()),
        ("B".to_string(), "Belgium".to_string()),
        ("BG".to_string(), "Bulgaria".to_string()),
        ("BIH".to_string(), "Bosnia and Herzegovina".to_string()),
        ("BY".to_string(), "Belarus".to_string()),
        ("CH".to_string(), "Switzerland".to_string()),
        ("CY".to_string(), "Cyprus".to_string()),
        ("CZ".to_string(), "Czech Republic".to_string()),
        ("D".to_string(), "Germany".to_string()),
        ("DK".to_string(), "Denmark".to_string()),
        ("E".to_string(), "Spain".to_string()),
        ("EST".to_string(), "Estonia".to_string()),
        ("F".to_string(), "France".to_string()),
        ("FIN".to_string(), "Finland".to_string()),
        ("FL".to_string(), "Liechtenstein".to_string()),
        ("FO".to_string(), "Faroe Islands".to_string()),
        ("GE".to_string(), "Georgia".to_string()),
        ("GR".to_string(), "Greece".to_string()),
        ("H".to_string(), "Hungary".to_string()),
        ("HR".to_string(), "Croatia".to_string()),
        ("I".to_string(), "Italy".to_string()),
        ("IRL".to_string(), "Ireland".to_string()),
        ("IS".to_string(), "Iceland".to_string()),
        ("KZ".to_string(), "Kazakhstan".to_string()),
        ("L".to_string(), "Luxembourg".to_string()),
        ("LT".to_string(), "Lithuania".to_string()),
        ("LV".to_string(), "Latvia".to_string()),
        ("M".to_string(), "Malta".to_string()),
        ("MD".to_string(), "Moldova".to_string()),
        ("MNE".to_string(), "Montenegro".to_string()),
        ("N".to_string(), "Norway".to_string()),
        ("NL".to_string(), "Netherlands".to_string()),
        ("NMK".to_string(), "North Macedonia".to_string()),
        ("P".to_string(), "Portugal".to_string()),
        ("PL".to_string(), "Poland".to_string()),
        ("RO".to_string(), "Romania".to_string()),
        ("RSM".to_string(), "San Marino".to_string()),
        ("RUS".to_string(), "Russia".to_string()),
        ("S".to_string(), "Sweden".to_string()),
        ("SK".to_string(), "Slovakia".to_string()),
        ("SLO".to_string(), "Slovenia".to_string()),
        ("SRB".to_string(), "Serbia".to_string()),
        ("TR".to_string(), "Turkey".to_string()),
        ("UA".to_string(), "Ukraine".to_string()),
        ("UK".to_string(), "United Kingdom".to_string()),
        ("V".to_string(), "Vatican City".to_string()),
    ]
}
