use ratatui::{
    crossterm::event::{self, Event},
    layout::Constraint,
    widgets::{Block, Borders, Padding},
};
use ratiform::{Form, FormLayout, builder::FormBuilder, custom_layout};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Field {
    Address,
    City,
    Email,
    Password,
    State,
    Zip,
}

fn main() -> std::io::Result<()> {
    let mut state = FormBuilder::new()
        .single_line(Field::Email, "Username")
        .single_line(Field::Password, "Password")
        .required("Password is required".to_owned())
        .masked()
        .single_line(Field::Address, "Address")
        .single_line(Field::City, "City")
        .single_line(Field::State, "State")
        .single_line(Field::Zip, "Zip")
        .build();

    // Email                   Password
    // _______________________ ____________________
    //
    // Address
    // ____________________________________________
    //
    // City                  State          Zip
    // _____________________ ______________ _______

    let grid_layout = custom_layout! {
        // Email | Password
        row [
            (Constraint::Length(15), Label(Field::Email)),
            (Constraint::Fill(1), Label(Field::Password)),
        ],
        row [
            (Constraint::Length(15), Value(Field::Email)),
            (Constraint::Fill(1), Value(Field::Password)),
        ],
        row [
            (Constraint::Length(15), Error(Field::Email)),
            (Constraint::Fill(1), Error(Field::Password)),
        ],

        // Address
        row [
            (Constraint::Fill(1), Label(Field::Address)),
        ],
        row [
            (Constraint::Fill(1), Value(Field::Address)),
        ],
        row [
            (Constraint::Fill(1), Error(Field::Address)),
        ],

        // City | State | Zip
        row [
            (Constraint::Length(15), Label(Field::City)),
            (Constraint::Length(15), Label(Field::State)),
            (Constraint::Fill(1), Label(Field::Zip)),
        ],
        row [
            (Constraint::Length(15), Value(Field::City)),
            (Constraint::Length(15), Value(Field::State)),
            (Constraint::Fill(1), Value(Field::Zip)),
        ],
        row [
            (Constraint::Length(15), Error(Field::City)),
            (Constraint::Length(15), Error(Field::State)),
            (Constraint::Fill(1), Error(Field::Zip)),
        ],
    };
    let form = Form::default().with_layout(FormLayout::Custom(grid_layout));

    ratatui::run(|terminal| -> std::io::Result<_> {
        loop {
            terminal.draw(|frame| {
                let area = frame
                    .area()
                    .centered(Constraint::Length(50), Constraint::Length(8));
                let block = Block::default()
                    .title(" Login ")
                    .borders(Borders::ALL)
                    .padding(Padding::uniform(1));
                let inner = block.inner(area);

                frame.render_widget(block, area);
                frame.render_stateful_widget(&form, inner, &mut state);

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
    })?;

    Ok(())
}
