# ratiform
[![CI](https://github.com/marc0x71/ratiform/actions/workflows/ci.yml/badge.svg)](https://github.com/marc0x71/ratiform/actions/workflows/ci.yml)

A small, composable, stateful form widget for [Ratatui](https://github.com/ratatui/ratatui).

Build forms with a typed field identity, keep the state in your application, and render them like any other Ratatui widget.

> **⚠️ Work in progress**
>
> `ratiform` is currently under active development. The API may change, and the project should be considered experimental for now.

`ratiform` provides a small, builder-based form component for [Ratatui](https://github.com/ratatui/ratatui), with keyboard navigation and a few basic input widgets.

The project is dual-licensed under the **MIT License** and the **Apache License 2.0** (see [License](#license) below).

## Design

```
             ratiform
                │
      ┌─────────┼─────────┐
      │         │         │
   Builder    State     Widget
      │         │         │
      └─────────┼─────────┘
                │
          your application
```

* The form doesn't own your application.
* The form doesn't own your data.
* The form doesn't decide what your UI looks like.
* The form handles input, focus, validation and rendering.

`FormBuilder` produces a `FormState<T>`, which your application owns for as long as the form is active — there is no hidden global state, no callback registry, nothing running in the background. `Form<T>` is a stateless `StatefulWidget`: you render it against that state exactly like you would any other Ratatui widget, in whichever `Rect`, frame, and event loop your application already has.

The `T` here is the type of your field identifiers — see [Field identifiers](#field-identifiers) below, which is where most of what makes `ratiform` different from other form widgets actually lives.

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

### Field identifiers

Every field is created together with an **identifier**, which is the first argument passed to `single_line()`, `checkbox()` and `select()`. `FormBuilder`, `FormState` and `Form` are all generic over the type of this identifier — it doesn't have to be a string or an integer, it can be your own `enum`:

```rust
#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    Nome,
    Cognome,
    Nazione,
    Termini,
}
```

This is what lets you match each submitted value back to the field that produced it (see [Retrieving the submitted values](#retrieving-the-submitted-values)) without relying on string keys: no risk of a typo in a field name going unnoticed until runtime, and the compiler will tell you if a `match` on the collected values forgets a variant. The examples in this README use a plain `enum` for this reason, even though any type works equally well — an integer, a `&'static str`, or anything else that fits your application.

Since fields are looked up by id (see [Reading and writing a single field](#reading-and-writing-a-single-field) below), `T` must implement `PartialEq`. `#[derive(PartialEq)]` — or `Hash, Eq, PartialEq` together, as in the `FormField` example above, needed if you also want to `collect()` into a `HashMap` — is enough for any plain enum or newtype.

With `FormField` in scope, a complete form looks like this:

```rust
let mut state = FormBuilder::new()
    .single_line(FormField::Nome, "Nome")
    .value("Mario")
    .validator(|value: &str| {
        (value.len() > 2)
            .then_some(())
            .ok_or_else(|| "Il nome deve avere una lunghezza maggiore di 2".to_owned())
    })
    .required()
    .single_line(FormField::Cognome, "Cognome")
    .value("Rossi")
    .validator(|value: &str| {
        (value.len() > 2)
            .then_some(())
            .ok_or_else(|| "Il cognome deve avere una lunghezza maggiore di 2".to_owned())
    })
    .validator(|value: &str| {
        (value.len() < 11)
            .then_some(())
            .ok_or_else(|| "Il cognome deve avere una lunghezza massima di 10".to_owned())
    })
    .required()
    .select(FormField::Nazione, "Paese")
    .values_ref(&[
        ("I", "Italia"),
        ("F", "Francia"),
        ("D", "Germania"),
    ])
    .selected(1)
    .height(5)
    .required()
    .checkbox(FormField::Termini, "Accetto i termini")
    .checked(false)
    .optional()
    .build();
```

Note the two `.validator(...)` calls chained on `Cognome`: both checks are kept, and are evaluated in that order (see [Validation](#validation) below).

A field can be marked invalid in two ways:

* **`required()`** — if the field is required and its current value is empty, it is automatically considered invalid, with a built-in message. This needs no extra configuration.
* **`validator(...)`** — a custom rule, `Fn(&str) -> Result<(), String> + 'static`. The closure receives the field's current value and returns `Err(message)` when the value is not acceptable. `validator(...)` can be called any number of times on the same field; each call adds one more check, evaluated in the order they were added.

The two interact like this:

| Field state | Result |
| --- | --- |
| `required()`, value empty | invalid, with the built-in required message — validators are **not** run |
| `optional()`, value empty | valid — validators are **not** run |
| any value, non-empty | each validator runs in order; the first one that returns `Err` wins and its message is shown; if all of them return `Ok`, the field is valid |

In other words, an empty value is never handed to a validator: `required()` decides on its own whether an empty field is acceptable, and only once a field actually has content do the custom validators get a say. This also means a validator doesn't need to special-case the empty string itself — that's `required()`/`optional()`'s job, not the validator's.

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

#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    Nome,
    Cognome,
    Nazione,
    Termini,
}

fn main() -> std::io::Result<()> {
    let result = ratatui::run(|terminal| -> std::io::Result<_> {
        let mut state = FormBuilder::new()
            .single_line(FormField::Nome, "Nome")
            .value("Mario")
            .validator(|value: &str| {
                (value.len() > 2)
                    .then_some(())
                    .ok_or_else(|| "Il nome deve avere una lunghezza maggiore di 2".to_owned())
            })
            .required()
            .single_line(FormField::Cognome, "Cognome")
            .value("Rossi")
            .validator(|value: &str| {
                (value.len() > 2)
                    .then_some(())
                    .ok_or_else(|| "Il cognome deve avere una lunghezza maggiore di 2".to_owned())
            })
            .validator(|value: &str| {
                (value.len() < 11)
                    .then_some(())
                    .ok_or_else(|| "Il cognome deve avere una lunghezza massima di 10".to_owned())
            })
            .required()
            .select(FormField::Nazione, "Paese")
            .values_ref(&[
                ("I", "Italia"),
                ("F", "Francia"),
                ("D", "Germania"),
            ])
            .selected(1)
            .height(5)
            .required()
            .checkbox(FormField::Termini, "Accetto i termini")
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
                    Form::new(),
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

### Reading and writing a single field

You don't have to wait for the form to be submitted to read or change a field's value — `FormState::value(&self, id: &T) -> Option<String>` returns the current value of the field with that id (`None` if no field has it), and `FormState::set_value(&mut self, id: &T, value: &str)` overwrites it:

```rust
if let Some(nome) = state.value(&FormField::Nome) {
    // ...
}

state.set_value(&FormField::Termini, "true");
```

A couple of details worth knowing before you reach for this:

* For a `Select` field, `set_value` matches against the **value** side of the pairs passed to `values_ref` (the first element, e.g. `"I"`, `"F"`, `"D"`), not the displayed label — the same convention `values_ref` itself already uses.
* For a `Checkbox` field, the string is parsed as a `bool` (`"true"` / `"false"`); anything else is treated as `false`.
* `set_value` triggers validation (see [Validation](#validation)) immediately, just like a keystroke would — so if the value you set fails `required()` or one of the field's validators, the field's error state is updated right away, before the next render.

### Focused field

`FormState::focus_field(&self) -> Option<&T>` returns the id of the field that currently has focus (`None` only if the form has no fields at all). It's handy for anything that needs to react to "which field is the user on right now" — for instance, showing contextual help for the focused field elsewhere on screen.

### Retrieving the submitted values

Once the form has reached `FormResult::Submitted` (or `FormResult::Cancelled`), the values entered by the user can be collected through `FormState::values()`, which returns an iterator of `(id, String)` pairs — one per field, paired with the identifier it was created with (see [Field identifiers](#field-identifiers)):

```rust
let values: HashMap<FormField, String> = state.values().collect();
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
