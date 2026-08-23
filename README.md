# ratiform

A simple form component for [Ratatui](https://github.com/ratatui/ratatui).

> **⚠️ Work in progress**
>
> `ratiform` is currently under active development. The API may change, and the project should be considered experimental for now.

`ratiform` provides a small, builder-based form component for [Ratatui](https://github.com/ratatui/ratatui), with keyboard navigation and a few basic input widgets.

The project is released under the **MIT License**.

## Installation

Since the project is still under development, it is currently intended to be used directly from GitHub.

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
ratiform = { git = "https://github.com/marc0x71/ratiform" }
```

`ratiform` currently depends on Ratatui `0.30`.

## Features

The following widgets are currently implemented:

### Single-line input

A simple single-line text input.

It supports:

* text insertion
* `Backspace`
* `Delete`
* cursor movement with `Left` / `Right`
* `Home` / `End`
* an initial value

Example:

```rust
.single_line("Nome")
    .value("Mario")
    .required()
```

### Checkbox

A boolean checkbox that can be toggled with the `Space` key.

```rust
.checkbox("Accetto i termini")
    .checked(false)
    .optional()
```

### Select

A selectable list of values.

The selected item can be changed using:

* `Up` / `Down`
* `Home` / `End`
* `PageUp` / `PageDown`

Example:

```rust
.select("Paese")
    .values_ref(&[
        ("I", "Italia"),
        ("F", "Francia"),
        ("D", "Germania"),
    ])
    .selected(1)
    .height(5)
    .required()
```

The first value in each pair is the value associated with the option, while the second is the text displayed to the user.

## Form builder

Forms are created using a builder API. Fields can be chained together and configured with common options such as:

* `required()`
* `optional()`
* `disabled()`
* `readonly()`
* `height()`

For example:

```rust
let mut state = FormBuilder::new()
    .single_line("Nome")
    .value("Mario")
    .required()
    .single_line("Cognome")
    .value("Rossi")
    .required()
    .select("Paese")
    .values_ref(&[
        ("I", "Italia"),
        ("F", "Francia"),
        ("D", "Germania"),
    ])
    .selected(1)
    .height(5)
    .required()
    .checkbox("Accetto i termini")
    .checked(false)
    .optional()
    .build();
```

## Usage

A complete example is available in [`examples/simple.rs`](examples/simple.rs).

The following is essentially the same example:

```rust
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
            .values_ref(&[
                ("I", "Italia"),
                ("F", "Francia"),
                ("D", "Germania"),
            ])
            .selected(1)
            .height(5)
            .required()
            .checkbox("Accetto i termini")
            .checked(false)
            .optional()
            .build();

        loop {
            terminal.draw(|frame| {
                let [area, _] =
                    Layout::vertical([
                        Constraint::Length(19),
                        Constraint::Fill(1),
                    ])
                    .areas(frame.area());

                frame.render_stateful_widget(
                    Form::default(),
                    area,
                    &mut state,
                );

                if let Some(position) = state.cursor_position() {
                    frame.set_cursor_position(position);
                }
            })?;

            if let Event::Key(key) = event::read()?
                && key.kind == ratatui::crossterm::event::KeyEventKind::Press
            {
                state.handle_input(key);

                match state.result() {
                    ratiform::FormResult::Submitted
                    | ratiform::FormResult::Cancelled => {
                        break Ok(());
                    }
                    ratiform::FormResult::Working => {}
                }
            }
        }
    })
}
```

### Keyboard navigation

The form handles a few global keys automatically:

| Key         | Action                     |
| ----------- | -------------------------- |
| `Tab`       | Move to the next field     |
| `Shift+Tab` | Move to the previous field |
| `Enter`     | Submit the form            |
| `Esc`       | Cancel the form            |

The keys specific to each widget are handled by the currently focused field.

For example, `Space` toggles a checkbox, while the arrow keys navigate a select field.

## Form result

The current state of the form can be inspected through `FormState::result()`:

```rust
match state.result() {
    ratiform::FormResult::Submitted => {
        // Form submitted
    }
    ratiform::FormResult::Cancelled => {
        // Form cancelled
    }
    ratiform::FormResult::Working => {
        // Still editing
    }
}
```

The current cursor position is also available through:

```rust
state.cursor_position()
```

which can be passed to Ratatui's `set_cursor_position()` when rendering a focused text field.

## Current status

This project is still in an early stage.

The current implementation is intentionally small and focused on providing a basic form abstraction for Ratatui. APIs, behaviour and rendering may change as the project evolves.

Some options are already exposed by the builder but are not yet fully implemented, so they should not necessarily be considered stable or functional at this stage.

Contributions, ideas and bug reports are welcome.

## License

`ratiform` is dual-licensed under the following licenses:

* [MIT License](LICENSE-MIT)
* [Apache License 2.0](LICENSE-APACHE)

You may choose to use `ratiform` under the terms of either license.
