 # ratiform
[![CI](https://github.com/marc0x71/ratiform/actions/workflows/ci.yml/badge.svg)](https://github.com/marc0x71/ratiform/actions/workflows/ci.yml)

A simple form component for [Ratatui](https://github.com/ratatui/ratatui).

> **⚠️ Work in progress**
>
> `ratiform` is currently under active development. The API may change, and the project should be considered experimental for now.

`ratiform` provides a small, builder-based form component for [Ratatui](https://github.com/ratatui/ratatui), with keyboard navigation and a few basic input widgets.

The project is dual-licensed under the **MIT License** and the **Apache License 2.0** (see [License](#license) below).

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
.single_line(1, "Nome")
    .value("Mario")
    .required()
```

The first argument is the field's **identifier** (see [Field identifiers](#field-identifiers) below).

### Checkbox

A boolean checkbox that can be toggled with the `Space` key.

```rust
.checkbox(4, "Accetto i termini")
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
.select(3, "Paese")
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

* `required()` / `optional()`
* `disabled()`
* `readonly()`
* `height()`
* `validator(...)`

For example:

```rust
let mut state = FormBuilder::new()
    .single_line(1, "Nome")
    .value("Mario")
    .validator(Box::new(|value: &str| {
        (value.len() > 2)
            .then_some(())
            .ok_or_else(|| "Il nome deve avere una lunghezza maggiore di 2".to_owned())
    }))
    .required()
    .single_line(2, "Cognome")
    .value("Rossi")
    .required()
    .select(3, "Paese")
    .values_ref(&[
        ("I", "Italia"),
        ("F", "Francia"),
        ("D", "Germania"),
    ])
    .selected(1)
    .height(5)
    .required()
    .checkbox(4, "Accetto i termini")
    .checked(false)
    .optional()
    .build();
```

### Field identifiers

Every field is created together with an **identifier** (`1`, `2`, `3`, `4` in the example above), which is the first argument passed to `single_line()`, `checkbox()` and `select()`. `FormBuilder`, `FormState` and `Form` are all generic over the type of this identifier, so it can be anything: an integer, a `&'static str`, a custom `enum`, ...

The identifier is what lets you match each submitted value back to the field that produced it — see [Retrieving the submitted values](#retrieving-the-submitted-values).

### Validation

Two independent checks can mark a field as invalid:

* **`required()`** — if the field is required and its current value is empty, it is automatically considered invalid. This needs no extra configuration.
* **`validator(...)`** — a custom rule, expressed as `Box<dyn Fn(&str) -> Result<(), String>>`. The closure receives the field's current value and returns `Err(message)` when the value is not acceptable. Only one validator per field is supported; calling `validator()` again replaces the previous one.

Validation runs **in real time**: every time a keystroke changes a field's value, that field is immediately re-checked, not just when the form is submitted. When a field is invalid, its error message is rendered in red to the right of the field.

While at least one field is invalid, pressing `Enter` has no effect: the form stays in `FormResult::Working` and is not submitted. Once every field passes validation, `Enter` submits the form as usual.

## Usage

A complete example is available in [`examples/simple.rs`](examples/simple.rs).

The following is essentially the same example:

```rust
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
            .validator(Box::new(|value: &str| {
                (value.len() > 2)
                    .then_some(())
                    .ok_or_else(|| "Il nome deve avere una lunghezza maggiore di 2".to_owned())
            }))
            .required()
            .single_line(2, "Cognome")
            .value("Rossi")
            .required()
            .select(3, "Paese")
            .values_ref(&[
                ("I", "Italia"),
                ("F", "Francia"),
                ("D", "Germania"),
            ])
            .selected(1)
            .height(5)
            .required()
            .checkbox(4, "Accetto i termini")
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
                && key.kind == event::KeyEventKind::Press
            {
                state.handle_input(key);

                match state.result() {
                    ratiform::FormResult::Submitted
                    | ratiform::FormResult::Cancelled => {
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
```

### Keyboard navigation

The form handles a few global keys automatically:

| Key         | Action                     |
| ----------- | -------------------------- |
| `Tab`       | Move to the next field     |
| `Shift+Tab` | Move to the previous field |
| `Enter`     | Submit the form, unless some field is currently invalid |
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

### Retrieving the submitted values

Once the form has reached `FormResult::Submitted` (or `FormResult::Cancelled`), the values entered by the user can be collected through `FormState::values()`, which returns an iterator of `(id, String)` pairs — one per field, paired with the identifier it was created with (see [Field identifiers](#field-identifiers)):

```rust
let values: HashMap<i32, String> = state.values().collect();
```

Note that `values()` consumes the `FormState`, so it's meant to be called once, after you're done using the form.

## Current status

This project is still in an early stage.

The current implementation is intentionally small and focused on providing a basic form abstraction for Ratatui. APIs, behaviour and rendering may change as the project evolves.

`required()`, `disabled()`, `readonly()` and `validator(...)` are now enforced, and submitted values can be retrieved through `FormState::values()`. Rendering and error reporting are still fairly basic (for instance, only single-line text fields can carry a custom validator's semantics meaningfully, and there is no built-in library of common validators yet), so behaviour and layout may still change as the project evolves.

Contributions, ideas and bug reports are welcome.

## License

`ratiform` is dual-licensed under the following licenses:

* [MIT License](LICENSE-MIT)
* [Apache License 2.0](LICENSE-APACHE)

You may choose to use `ratiform` under the terms of either license.

## A note about AI

I used AI to help me write some of the tests and documentation for this project. Writing tests can be a bit tedious, and English isn't my native language, so AI has been a useful tool to speed things up and improve the documentation.

I still review, adapt, and run the generated tests, but I prefer to be transparent about how AI was used in this project.

