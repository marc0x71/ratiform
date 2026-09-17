# ratiform
[![CI](https://github.com/marc0x71/ratiform/actions/workflows/ci.yml/badge.svg)](https://github.com/marc0x71/ratiform/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ratiform.svg)](https://crates.io/crates/ratiform)

**A small, composable, stateful form widget for [Ratatui](https://ratatui.rs/), with typed field identifiers and application-owned data.**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Field {
    Name,
    Email,
    Country,
    Terms,
}

let mut state = FormBuilder::new()
    .single_line(Field::Name, "Name")
    .single_line(Field::Email, "Email")
    .select(Field::Country, "Country")
    .values_ref(&[("IT", "Italy"), ("FR", "France"), ("DE", "Germany")])
    .checkbox(Field::Terms, "I accept the terms")
    .build().unwrap();
```

The field identity is a real Rust type — `state.value(&Field::Email)`, not `state.value("email")`. No string keys, no JSON round-trip, no form-specific data model.

The project is dual-licensed under the **MIT License** and the **Apache License 2.0** (see [License](#license) below).

<img width="1000" height="600" alt="ratiform" src="https://github.com/user-attachments/assets/8015202a-1613-4621-b599-f46d2f60fa12" />

## Why ratiform?

I started this while building a different TUI application that needed a couple of text inputs. At first I wrote them the old-fashioned way: a `String` in my state, rendering and key handling done by hand, field by field. Fine for one input, a bit repetitive for two.

Then I needed a third field, and hand-rolling focus management — which field is active, what `Tab` should do, how to keep three separate pieces of state in sync — stopped being worth it. [`tui-input`](https://github.com/sayanarijit/tui-input) looked like the natural next step, but it manages a single field; grouping several of them and moving focus between them was still on me.

So I went looking for something closer to an actual *form*: own the fields, own the focus, hand me back what the user typed. I found [`ratatui-form`](https://github.com/DavidLiedle/ratatui-form), which does exactly that, and more than `ratiform` does today. What made me pause was how it hands the values back — serialized to JSON, keyed by field name as a string. I would have had to turn my own data into a serialization format, only to deserialize it again, to get data whose shape I already knew at compile time.

That's the itch `ratiform` scratches: field identifiers stay real Rust types from the moment you create a field to the moment you read its value back. No string keys, no serialization round-trip, and no form framework trying to own your application's data.

I doubt I'm the only one this has bothered — hence this project. 😅

## Design

```
┌──────────────────────────────┐
│       your application       │
│                              │
│   domain model, business     │
│   logic, persistence         │
└──────────────┬───────────────┘
               │ field values, typed by you
               ▼
┌──────────────────────────────┐
│           ratiform           │
│                              │
│  input · focus · validation  │
│  navigation · dirty state    │
└──────────────┬───────────────┘
               │ a StatefulWidget
               ▼
┌──────────────────────────────┐
│            Ratatui           │
└──────────────────────────────┘
```

* The form doesn't own your application, or your data — it hands values back as plain `String`s tied to *your* id type, and never guesses what they mean.
* The form doesn't decide what your UI looks like — `Form<T>` is a stateless `StatefulWidget`, composable with any other widget (see [`examples/login-form.rs`](examples/login-form.rs), which wraps it in a titled `Block`).
* `FormBuilder` produces a `FormState<T>`, which your application owns for as long as the form is active — no hidden global state, no callback registry.

## Installation

```bash
cargo add ratiform
```

`ratiform` currently depends on Ratatui `0.30`.

If you want to use the latest development version, you can depend directly on GitHub:

```toml
[dependencies]
ratiform = { git = "https://github.com/marc0x71/ratiform" }
```

## Quick start

```rust
use ratatui::{
    crossterm::event::{self, Event},
    layout::Constraint,
    widgets::{Block, Borders, Padding},
};
use ratiform::{Form, builder::FormBuilder};

#[derive(Debug, Hash, Eq, PartialEq)]
enum Field {
    Username,
    Password,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = FormBuilder::new()
        .single_line(Field::Username, "Username")
        .required("Username is required".to_owned())
        .single_line(Field::Password, "Password")
        .masked()
        .required("Password is required".to_owned())
        .build()?;

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
                frame.render_stateful_widget(Form::default(), inner, &mut state);

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
```

The exact same code lives in [`examples/quickstart.rs`](examples/quickstart.rs).

## Learn more

The [full tutorial](docs/tutorial.md) covers every field kind, validation,
normalizing values, layout (including the `Custom` grid layout), theming,
keyboard navigation, reading and driving the form at runtime, and how
`build()` reports errors — one chapter per feature, with runnable examples
linked throughout. The generated API docs (`cargo doc --open`) remain the
source of truth for exact method signatures.

## Current status

This project is still pre-1.0. As with any 0.x crate, minor version bumps may include breaking changes — check the [changelog](CHANGELOG.md) before upgrading. The core design — typed field identifiers, the builder/state/widget split, validation — is stable; newer field kinds are more likely to see API adjustments.

Contributions, ideas and bug reports are welcome. If you're thinking about adding a new field kind, see [`docs/adding-a-widget.md`](docs/adding-a-widget.md) for the wiring points and conventions the existing five already follow.

## License

`ratiform` is dual-licensed under the [MIT License](LICENSE-MIT) and the [Apache License 2.0](LICENSE-APACHE). You may choose either.

## A note about AI

I used AI to help me write some of the tests and documentation for this project. Writing tests can be a bit tedious, and English isn't my native language, so AI has been a useful tool to speed things up and improve the documentation.

I still review, adapt, and run the generated tests, but I prefer to be transparent about how AI was used in this project.
