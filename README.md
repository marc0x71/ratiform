# ratiform
[![CI](https://github.com/marc0x71/ratiform/actions/workflows/ci.yml/badge.svg)](https://github.com/marc0x71/ratiform/actions/workflows/ci.yml)

A small, composable, stateful form widget for [Ratatui](https://github.com/ratatui/ratatui).

Build forms with a typed field identity, keep the state in your application, and render them like any other Ratatui widget.

> **⚠️ Work in progress**
>
> `ratiform` is currently under active development. Breaking changes are still possible before a stable release, so pin a specific commit if you depend on it.

`ratiform` provides a small, builder-based form component for [Ratatui](https://github.com/ratatui/ratatui), with keyboard navigation and a few basic input widgets.

The project is dual-licensed under the **MIT License** and the **Apache License 2.0** (see [License](#license) below).

## Why ratiform?

I started this while building a different TUI application that needed a couple of text inputs. At first I wrote them the old-fashioned way: a `String` in my state, rendering and key handling done by hand, field by field. Fine for one input, a bit repetitive for two.

Then I needed a third field, and hand-rolling focus management — which field is active, what `Tab` should do, how to keep three separate pieces of state in sync — stopped being worth it. [`tui-input`](https://github.com/sayanarijit/tui-input) looked like the natural next step, but it manages a single field; grouping several of them and moving focus between them was still on me.

So I went looking for something closer to an actual *form*: own the fields, own the focus, hand me back what the user typed. I found [`ratatui-form`](https://github.com/DavidLiedle/ratatui-form), which does exactly that, and more than `ratiform` does today. What made me pause was how it hands the values back — serialized to JSON, keyed by field name as a string. I would have had to turn my own data into a serialization format, only to deserialize it again, to get data whose shape I already knew at compile time.

That's the itch `ratiform` scratches: field identifiers stay real Rust types from the moment you create a field to the moment you read its value back. No string keys, no serialization round-trip, and no form framework trying to own your application's data.

I doubt I'm the only one this has bothered — hence this project. 😅

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

`FormBuilder` produces a `FormState<T>`, which your application owns for as long as the form is active — there is no hidden global state, no callback registry, nothing running in the background. `Form<T>` is a stateless `StatefulWidget`: you render it against that state exactly like you would any other Ratatui widget, in whichever `Rect`, frame, and event loop your application already has. That includes composing it with other widgets — wrapping it in a titled `Block`, for instance, needs no support from `ratiform` at all (see [`examples/login-form.rs`](examples/login-form.rs)).

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
* masking (for password-like fields)
* a placeholder shown while the value is empty

Example:

```rust
.single_line(1, "Nome")
    .value("Mario")
    .required("Il nome è obbligatorio".to_owned())
```

The first argument is the field's **identifier** (see [Field identifiers](#field-identifiers) below).

#### Masked fields

For a password-style field, use `masked()` (the character typed is replaced with `*`) or `masked_with(c)` to pick your own mask character:

```rust
.single_line(5, "Password")
    .masked_with('•')
    .required("La password non può essere vuota".to_owned())
```

Masking only changes what's drawn on screen — the field's real value, the required check, `validator(...)` and everything read back through `value()`/`values()` all still see (and act on) what the user actually typed, not the mask. The cursor stays correctly aligned regardless of the mask character or of multi-byte characters in the value, since the displayed string always has exactly as many characters as the real one.

#### Placeholder

`placeholder(text)` shows a hint whenever the field's value is empty:

```rust
.single_line(1, "Nome")
    .placeholder("Inserisci il nome")
    .required("Il nome è obbligatorio".to_owned())
```

The placeholder disappears as soon as the user types anything, and is rendered with its own style (see [Theming](#theming)) so it doesn't get mistaken for real content. It's drawn in plain text even on a `masked_with(...)` field — masking a hint that isn't real input would just make it unreadable — and, like the mask itself, it has no effect on validation: a required field showing a placeholder is still empty as far as the required check and `values()` are concerned.

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
    .required("Seleziona un paese".to_owned())
```

The first value in each pair is the value associated with the option, while the second is the text displayed to the user.

## Form builder

Forms are created using a builder API. Fields can be chained together and configured with common options such as:

* `required(message)` / `optional()`
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
use ratiform::validators;

let mut state = FormBuilder::new()
    .single_line(FormField::Nome, "Nome")
    .value("Mario")
    .validator(validators::min_length(
        2,
        "Il nome deve avere una lunghezza di almeno 2 caratteri".to_owned(),
    ))
    .required("Il nome è obbligatorio".to_owned())
    .single_line(FormField::Cognome, "Cognome")
    .value("Rossi")
    .validator(validators::min_length(
        2,
        "Il cognome deve avere una lunghezza di almeno 2 caratteri".to_owned(),
    ))
    .validator(validators::max_length(
        10,
        "Il cognome deve avere una lunghezza massima di 10 caratteri".to_owned(),
    ))
    .select(FormField::Nazione, "Paese")
    .values_ref(&[
        ("I", "Italia"),
        ("F", "Francia"),
        ("D", "Germania"),
    ])
    .selected(1)
    .height(5)
    .checkbox(FormField::Termini, "Accetto i termini")
    .checked(false)
    .optional()
    .build();
```

Note the two `.validator(...)` calls chained on `Cognome`: both checks are kept, and are evaluated in that order (see [Validation](#validation) below). Also note that `Cognome` and `Paese` never call `.required(...)` at all — every field is required by default (with a built-in message), so `.required(message)` is only needed when you want your *own* message; `.optional()` is what actually changes behavior, by opting a field out of the required check entirely.

A field can be marked invalid in two ways:

* **Being required** — every field is required by default, and an empty required field is invalid with a built-in message. Call `.required(message)` to use your own message instead of the built-in one, or `.optional()` to opt the field out of this check entirely. Either way, this doesn't need `validator(...)`: internally, `.required(message)` is itself backed by a `Validator` (`ratiform::validators::required`, see [Built-in validators](#built-in-validators)), just kept in its own slot rather than the general list below.
* **`validator(...)`** — a custom rule, `Fn(&str) -> Result<(), String> + 'static`. The closure receives the field's current value and returns `Err(message)` when the value is not acceptable. `validator(...)` can be called any number of times on the same field; each call adds one more check, evaluated in the order they were added.

The two interact like this:

| Field state | Result |
| --- | --- |
| required (default, or `.required(message)`), value empty | invalid, with the required message — validators are **not** run |
| `optional()`, value empty | valid — validators are **not** run |
| any value, non-empty | each validator runs in order; the first one that returns `Err` wins and its message is shown; if all of them return `Ok`, the field is valid |

In other words, an empty value is never handed to a validator: the required check decides on its own whether an empty field is acceptable, and only once a field actually has content do the custom validators get a say. This also means a validator doesn't need to special-case the empty string itself — that's the required check's job, not the validator's.

Validation runs **in real time**: every time a keystroke changes a field's value, that field is immediately re-checked, not just when the form is submitted — and a field is validated once up front too, as soon as the form is built, so a field that starts out invalid (an initial `.value(...)` too short, or a required field left empty) shows its error from the very first render, not only after the user touches it. When a field is invalid, its error message is rendered next to the field, styled with `FormStyle`'s `error` (red and bold by default — see [Theming](#theming) to change it).

While at least one field is invalid, pressing `Enter` has no effect: the form stays in `FormResult::Working` and is not submitted. Once every field passes validation, `Enter` submits the form as usual.

### Built-in validators

`validator(...)` accepts any closure, so you can always write your own check from scratch — but the `ratiform::validators` module ships a handful of common ones, each taking the error message to show and returning a ready-to-use `Validator`:

| Function | Checks that the value... | Notes |
| --- | --- | --- |
| `required(message)` | is not empty | this is what `.required(message)` uses internally — see [Validation](#validation) above. Rarely called directly, but it's a plain `Validator` like the rest, so you can compose it yourself if needed |
| `min_length(len, message)` | has at least `len` characters | inclusive — `min_length(5, ..)` accepts a 5-character value |
| `max_length(len, message)` | has at most `len` characters | inclusive — `max_length(10, ..)` accepts a 10-character value |
| `is_numeric(message)` | consists only of ASCII digits (`0`–`9`) | no sign, no decimal point — `"-5"` and `"3.14"` are both rejected |
| `alphabetic(message)` | consists only of letters | Unicode-aware — accented letters like `à` count as letters |
| `alphanumeric(message)` | consists only of letters and digits | same Unicode-awareness as `alphabetic` |
| `no_whitespace(message)` | contains no whitespace (spaces, tabs, newlines) | |
| `parsable::<T>(message)` | parses as a `T` via `T: FromStr` | see below — needs the turbofish |

`min_length`/`max_length` count Unicode characters, not bytes — `min_length(5, ..)` correctly accepts `"città"` (5 characters, 6 bytes in UTF-8). All the shape-based validators above (everything except `required` and `parsable`) pass on an empty string: they check *shape*, and an empty value has no character that violates the rule. In practice this rarely matters, because inside a `Field`, the required check already decides whether an empty value is acceptable before any of these run (see the table in [Validation](#validation)) — but it's worth knowing if you call one of these functions directly, outside of a `Field`.

`parsable` is different from the rest: since `T` only appears in the function body, not in `Validator`'s return type, the compiler can't infer it — you always need the turbofish, `parsable::<i32>(message)`, `parsable::<f64>(message)`, and so on. Being generic over any `T: FromStr` also means it isn't limited to numbers: any type with a `FromStr` implementation works, including ones from other crates. For example, `chrono::NaiveDate` implements `FromStr` for the ISO `YYYY-MM-DD` format, so `parsable::<chrono::NaiveDate>("data non valida".to_owned())` gives you a correct date validator — leap years included — without `ratiform` itself depending on `chrono`. That dependency, if you want it, lives in your own `Cargo.toml`, not in `ratiform`'s.

For anything these don't cover — a different date format, a regex pattern, a check across multiple fields — `.validator(...)` still takes a plain closure exactly as before; the built-ins are a convenience on top of that, not a replacement for it.

## Usage

A complete example is available in [`examples/simple-typed.rs`](examples/simple-typed.rs).

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
            .single_line(FormField::Cognome, "Cognome")
            .value("Rossi")
            .select(FormField::Nazione, "Paese")
            .values_ref(&[
                ("I", "Italia"),
                ("F", "Francia"),
                ("D", "Germania"),
            ])
            .selected(1)
            .height(5)
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

`Form::default()` renders with the built-in theme; see [Theming](#theming) below for how to use your own.

## More examples

A few more programs live in [`examples/`](examples), each built around a specific situation rather than a feature tour:

* [`examples/simple.rs`](examples/simple.rs) — the same form as above, rendered with a custom `FormStyle` (see [Theming](#theming)), plus a couple of debug fields exercising `set_value`/`focused_field`.
* [`examples/login-form.rs`](examples/login-form.rs) — a login screen: a required username with a minimum length, a masked password with a custom strength check, an optional "remember me" checkbox, and the form itself rendered inside a titled, centered `Block` rather than filling the whole screen.
* [`examples/connections.rs`](examples/connections.rs) — a connection settings form: `validators::parsable::<u16>` to validate a port number, a `Select` for the protocol, and a mix of required and optional fields side by side.
* [`examples/anagrafica.rs`](examples/anagrafica.rs) — forcing a field to stay uppercase or lowercase as the user types, using `value()`/`set_value()` in the event loop rather than any dedicated feature of the library.

Run any of them with `cargo run --example <name>` (e.g. `cargo run --example login-form`).

## Theming

By default, `Form::default()` renders with a built-in gray/bold/reversed color scheme. To use your own, build a [`ratiform::style::FormStyle`](src/style.rs) and hand it to `Form::with_style(...)` instead of calling `Form::default()`:

```rust
use ratatui::style::{Color, Style};
use ratiform::style::{FieldStyle, FormStyle};

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
```

```rust
frame.render_stateful_widget(Form::with_style(my_style()), area, &mut state);
```

`FormStyle` groups five areas:

* **`label`** — the field's caption, on the left.
* **`value`** — the field's own content: the text color of a `SingleLine` field, a `Select` field's list items, and a `Checkbox`'s `[✓]` / `[ ]` glyph.
* **`highlight`** — emphasis for whatever is "active right now": the background box behind a `SingleLine` field, and the currently selected row of a `Select` list.
* **`error`** — the validation error message shown under an invalid field. The error is rendered right-aligned in an area sized to the message itself, so a background color set here (like the white-on-red in the example above) hugs just the text rather than filling the whole row.
* **`placeholder`** — a `SingleLine` field's [placeholder text](#placeholder), shown in place of `value`'s style while the field is empty.

Each of `label`, `value` and `highlight` is a [`FieldStyle`](src/style.rs), carrying one `Style` per state: `normal`, `focused`, `disabled`, `readonly`. You don't need to pick which one applies yourself — `FieldOptions` resolves it for you, with `disabled` taking priority over `readonly`, which takes priority over `focused`. `error` and `placeholder` are plain `Style`s instead — unlike the other three, they're not tied to focus/disabled/readonly, they simply apply whenever an error or a placeholder is shown.

A runnable variant of the earlier example with a custom `FormStyle`, placeholders on every text field, a masked `Password` field, and a couple of debug fields exercising `set_value`/`focused_field`, is in [`examples/simple.rs`](examples/simple.rs).

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
* `set_value` triggers validation (see [Validation](#validation)) immediately, just like a keystroke would — so if the value you set fails the required check or one of the field's validators, the field's error state is updated right away, before the next render.

### Focused field

`FormState::focused_field(&self) -> Option<&T>` returns the id of the field that currently has focus (`None` only if the form has no fields at all). It's handy for anything that needs to react to "which field is the user on right now" — for instance, showing contextual help for the focused field elsewhere on screen.

### Dirty state and resetting

Every field remembers the value it was built with — whatever `.value(...)`, `.checked(...)` or `.selected(...)` set, or the default if none of those were called. "Dirty" means the current value no longer matches that original one:

* **`FormState::is_dirty(&self) -> bool`** — `true` if at least one field has been changed since the form was built.
* **`FormState::is_field_dirty(&self, id: &T) -> Option<bool>`** — the same check for a single field, `None` if no field has that id (same convention as `value()`).
* **`FormState::reset(&mut self)`** — restores every field to its original value, and re-validates each one against it, so the error state after a reset matches the restored values rather than whatever was on screen a moment before.

```rust
if state.is_dirty() {
    // warn the user before discarding their changes, for example
}

state.reset();
```

`set_value(...)` counts as a change here too, exactly as if the user had typed it — it updates a field's dirty state the same way a keystroke would, since it goes through the same underlying value.

### Retrieving the submitted values

Once the form has reached `FormResult::Submitted` (or `FormResult::Cancelled`), the values entered by the user can be collected through `FormState::values()`, which returns an iterator of `(id, String)` pairs — one per field, paired with the identifier it was created with (see [Field identifiers](#field-identifiers)):

```rust
let values: HashMap<FormField, String> = state.values().collect();
```

Note that `values()` consumes the `FormState`, so it's meant to be called once, after you're done using the form.

## Current status

This project is still in an early stage.

The current implementation is intentionally small and focused on providing a basic form abstraction for Ratatui. APIs, behaviour and rendering may change as the project evolves.

The required check, `disabled()`, `readonly()` and `validator(...)` are now enforced, submitted values can be retrieved through `FormState::values()`, rendering can be themed through `FormStyle` (see [Theming](#theming)), a handful of common checks are available in `ratiform::validators` (see [Built-in validators](#built-in-validators)), and forms can track unsaved changes and roll them back through `is_dirty()`/`is_field_dirty()`/`reset()` (see [Dirty state and resetting](#dirty-state-and-resetting)). Behaviour and layout may still change as the project evolves.

Contributions, ideas and bug reports are welcome.

## License

`ratiform` is dual-licensed under the following licenses:

* [MIT License](LICENSE-MIT)
* [Apache License 2.0](LICENSE-APACHE)

You may choose to use `ratiform` under the terms of either license.

## A note about AI

I used AI to help me write some of the tests and documentation for this project. Writing tests can be a bit tedious, and English isn't my native language, so AI has been a useful tool to speed things up and improve the documentation.

I still review, adapt, and run the generated tests, but I prefer to be transparent about how AI was used in this project.
