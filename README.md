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

> **⚠️ Work in progress**
>
> `ratiform` is currently under active development. Breaking changes are still possible before a stable release, so pin a specific commit if you depend on it.

The project is dual-licensed under the **MIT License** and the **Apache License 2.0** (see [License](#license) below).

<img width="1000" height="600" alt="ratiform" src="https://github.com/user-attachments/assets/1652a596-cbf9-4cd1-b58c-cd9f2815a0f7" />

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

The `T` here is the type of your field identifiers — see [Field identifiers](#field-identifiers), which is where most of what makes `ratiform` different from other form widgets actually lives.

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

The exact same code lives in [`examples/quickstart.rs`](examples/quickstart.rs) — see [More examples](#more-examples) below for a few other complete programs, closer to a real application than this one.

## Features

### Single-line input

* text insertion, `Backspace`/`Delete`, cursor movement (`Left`/`Right`/`Home`/`End`)
* an initial value, and a placeholder shown while the value is empty
* masking for password-like fields (`masked()`/`masked_with(c)`) — cosmetic only, validation and `value()` still see what was actually typed
* restricting which characters can be typed at all (`alphabet(chars)`), and/or rewriting the value into a canonical form as it's typed (`normalizer(...)`, e.g. forcing uppercase) — see [Normalizing values](#normalizing-values)

```rust
.single_line(Field::Name, "Name")
    .value("Mario")
    .placeholder("Enter your name")
    .required("Name is required".to_owned())
```

### Checkbox

A boolean, toggled with `Space`:

```rust
.checkbox(Field::Terms, "I accept the terms")
    .checked(false)
    .optional()
```

### Select

A list of `(value, label)` pairs, navigated with `Up`/`Down`/`Home`/`End`/`PageUp`/`PageDown`. `value()` returns the first element of the pair, not the label shown on screen:

```rust
.select(Field::Country, "Country")
    .values_ref(&[("IT", "Italy"), ("FR", "France"), ("DE", "Germany")])
    .selected(1)
    .height(5)
```

Two options sharing the same value make `build()` fail too — see [Duplicate ids and values](#duplicate-ids-and-values).

### Text area

Multi-line text, with the same insertion/deletion/placeholder support as single-line input, plus `Up`/`Down`, scrolling, and `PageUp`/`PageDown`. `Home`/`End` jump to the start/end of the current visual line; `Ctrl+Home`/`Ctrl+End` jump to the start/end of the whole text. Long lines wrap at the character level, not at word boundaries (the same default `vim` uses).

```rust
.text_area(Field::Notes, "Notes")
    .placeholder("Write here...")
    .height(5)
```

Since `Enter` inserts a newline instead of submitting the form, submitting while a `TextArea` has focus needs `Ctrl+Enter` — see [Keyboard navigation](#keyboard-navigation).

## Form builder

Fields are chained together and configured with options shared across every kind: `required(message)`/`optional()`, `disabled()`, `readonly()`, `hide()`/`show()`, `height()`, `validator(...)`, `normalizer(...)`. Full details on each are in the generated docs (`cargo doc --open`) — the sections below cover the ones with enough behavior to be worth a walkthrough.

### Label width

Every field shares a single label column, sized automatically to fit the widest label across all of them, capped to a third of the available area. A label too long for that cap wraps onto a second line rather than pushing the shared column — and every other field's label with it — off to the side. Wrapping only affects that field's own row height; the column width stays the same for every field regardless of whether its own label needed to wrap. Wrapping breaks on whole words, the same way Ratatui's own text wrapping does; a single word longer than the column still isn't split.

Call `.label_width(n)` on the builder to fix the column at `n` characters instead of computing it automatically. An explicit width is **not** capped — if you ask for more than the terminal can comfortably show, you get exactly what you asked for:

```rust
.single_line(1, "Name")
    .label_width(20)
```

`FormState::label_width(&mut self, width: u16)` sets the same thing after the form has already been built, for cases where the right width is only known once the form is running (in response to a resize, for instance).

### Layout

By default, every field's label and value sit side by side on the same row — [`FormLayout::Horizontal`](src/layout/mod.rs):

```
Name       [Mario_____________]
Email      [mario@example.com_]
```

`FormLayout::Stacked` puts the label above the value instead, each on its own row, useful when the terminal is too narrow for a comfortable label column:

```
Name
[Mario_____________]

Email
[mario@example.com_]
```

Set it with `.with_layout(...)` on `Form`, the same way you'd set a custom style with `.with_style(...)`:

```rust
frame.render_stateful_widget(
    Form::default().with_layout(FormLayout::Stacked),
    area,
    &mut state,
);
```

`Form` is rebuilt fresh every frame, so nothing stops you from picking the layout based on the current area — [`examples/layouts.rs`](examples/layouts.rs) does exactly that, switching to `Stacked` once the terminal gets too narrow, reacting to a resize with no extra code to handle the resize itself. `label_width` only means something in `Horizontal`; `Stacked` ignores it, since there's no shared column to size.

#### Custom layout

`FormLayout::Custom` gives up automatic arrangement for full control: an explicit grid of rows, and inside each row, columns — each cell either empty (a spacer) or one of a field's `Label`, `Value`, or `Error`. Useful for anything `Horizontal`/`Stacked` can't express — here, `Name` and `Email` sharing a row instead of each getting one of their own:

```
Name              Email
[Mario_________]  [mario@example.com_]
```

```rust
use ratatui::layout::Constraint;
use ratiform::{Form, FormLayout, custom_layout, layout::custom::CustomLayout};

let layout = custom_layout! {
    row [
        (Constraint::Length(15), Label(Field::Name)),
        (Constraint::Fill(1), Label(Field::Email)),
    ],
    row [
        (Constraint::Length(15), Value(Field::Name)),
        (Constraint::Fill(1), Value(Field::Email)),
    ],
    row [
        (Constraint::Length(15), Error(Field::Name)),
        (Constraint::Fill(1), Error(Field::Email)),
    ],
};

frame.render_stateful_widget(
    Form::default().with_layout(FormLayout::Custom(layout)),
    area,
    &mut state,
);
```

The same grid can be built imperatively with `CustomLayout::builder()` — a fluent `.row()...label()/value()/error()/empty()...build()` — when the rows aren't known until runtime. `Label` and `Error` both wrap onto multiple lines if the column is too narrow, the same as each other — `Error`'s row reserves at least one line even when the field currently has no error, growing to fit whatever message is actually there, so the layout doesn't jump the moment one appears. Adjacent cells get a small horizontal gap by default (`CustomLayout::with_column_gap` to change it).

Each grid row scrolls independently — unlike `Horizontal`/`Stacked`, where a field's label, value, and error always move together, a `Custom` layout scrolls whichever row currently has focus into view, which can leave a distant `Label`/`Error` row scrolled out of sight. Keep a field's cells on nearby rows if you want them to move together.

A cell whose id doesn't match any of the form's actual fields — a typo, or a layout built separately from the fields it's meant to describe — draws nothing and reports no error; it's silently treated as an empty cell.

See [`examples/custom.rs`](examples/custom.rs) for a runnable version.

### Field identifiers

Every field is created together with an **identifier** — the first argument to `single_line()`, `checkbox()`, `select()` and `text_area()`. `FormBuilder`, `FormState` and `Form` are all generic over its type (it needs `PartialEq`, plus `Hash`/`Eq` if you want to `collect()` results into a `HashMap`) — it doesn't have to be a string or an integer, it can be your own `enum`:

```rust
#[derive(Debug, Hash, Eq, PartialEq)]
enum FormField {
    FirstName,
    LastName,
    Country,
    Terms,
}

let mut state = FormBuilder::new()
    .single_line(FormField::FirstName, "First name")
    .value("Mario")
    .validator(validators::min_length(2, "Too short".to_owned()))
    .required("First name is required".to_owned())
    .single_line(FormField::LastName, "Last name")
    .select(FormField::Country, "Country")
    .values_ref(&[("IT", "Italy"), ("FR", "France"), ("DE", "Germany")])
    .checkbox(FormField::Terms, "I accept the terms")
    .optional()
    .build().unwrap();
```

This is what lets you match each submitted value back to the field that produced it, at compile time — no risk of a typo in a field name going unnoticed until runtime. Note that `LastName`/`Country` never call `.required(...)`: every field is required by default (with a built-in message), so `.required(message)` is only needed for your *own* message — `.optional()` is what actually opts a field out. Two fields sharing the same id make `build()` fail at runtime instead — see [Duplicate ids and values](#duplicate-ids-and-values).

### Duplicate ids and values

`build()` returns `Result<FormState<T>, BuildError>`, not `FormState<T>` directly — it fails if two fields share the same id, or if a `Select` field has two options with the same value. Both are checked as each field builder finishes, so the error you get is always the *first* one found while building the chain, even though later field builders still run to completion.

```rust
let result = FormBuilder::new()
    .single_line(Field::Name, "Name")
    .single_line(Field::Name, "Name again") // same id twice
    .build();

assert!(result.is_err());
```

This is a development-time mistake to catch, not something you'd handle differently at runtime — `.build()?` (with a `Box<dyn std::error::Error>` return type, as in every example) or `.build().unwrap()` are both reasonable, depending on whether `main` itself returns a `Result`.

### Validation

A field is invalid in two ways:

* **Being required** — every field is required by default; an empty required field is invalid, with a built-in or custom (`.required(message)`) message.
* **`validator(...)`** — `Fn(&str) -> Result<(), String>`, called any number of times on the same field, evaluated in the order added.

| Field state | Result |
| --- | --- |
| required, value empty | invalid, required message — validators don't run |
| `optional()`, value empty | valid — validators don't run |
| any value, non-empty | validators run in order; first `Err` wins, otherwise valid |

An empty value is never handed to a validator — the required check decides on its own whether it's acceptable, so a validator never needs to special-case the empty string.

Validation runs on every keystroke, on `set_value`, and once up front when the form is built, so a field that starts out invalid shows its error from the very first render. While any field is invalid, `Enter` doesn't submit the form.

`disabled()`/`readonly()` don't skip validation — and since every field is required by default, disabling one with no initial value and no `.optional()` leaves it permanently invalid *and* permanently unreachable, since `Tab`/`BackTab` skip disabled fields. Call `.optional()` on any field you disable without giving it a value.

### Field visibility

`hide()` on any field builder starts it hidden; `.show()` starts it visible (the default). At runtime, `FormState::set_visible(&id, bool)` toggles it, and `is_field_visible(&id)` reads it back (`None` if the id doesn't exist).

A hidden field draws nothing — label, value, and error alike — in every layout, and is skipped by `Tab`/`Shift+Tab` just like a `disabled()` field. In `Horizontal`/`Stacked`, hiding a field also reclaims its space automatically, since each field owns its own rows. In `Custom`, a hidden field's cells are drawn as empty rather than removed from the grid — the same as [a cell referencing an unknown id](#custom-layout) — so surrounding cells don't reflow into the freed space. Build a different `CustomLayout` for that state and swap it in with `.with_layout(...)` (rebuilt every frame anyway) if you want the space reclaimed instead.

On `CustomLayout` a field on rows of its own — no cell shared with another field — disappears entirely when hidden, since every cell on those rows resolves to zero height together; the rows below shift up to fill the gap. A field sharing a row with a still-visible one only clears its own cell, leaving the row's reserved width in place.

Unlike `disabled()`/`readonly()` (see above), a hidden field is also excluded from validation, so a required-but-empty field doesn't block submission while hidden.

### Normalizing values

Where `validator(...)` judges an already-typed value, `normalizer(...)` rewrites it into a canonical form before validation runs on it:

```rust
.single_line(FormField::CodiceFiscale, "Codice fiscale")
    .normalizer(|value: &str| value.to_uppercase())
    .validator(validators::max_length(16, "Troppo lungo".to_owned()))
```

It runs on every keystroke, on `set_value`, and on the initial value — a field's value is never seen (by validators, `is_dirty()`, `values()`) in anything other than its normalized form. Only one `normalizer` is kept per field; calling it again replaces the previous one. On a `SingleLine` field it pairs naturally with `alphabet(...)`: `alphabet` rejects a character outright, `normalizer` rewrites one that was allowed in.

### Built-in validators

`ratiform::validators` ships common checks, each taking the error message and returning a ready-to-use `Validator`: `required`, `min_length`/`max_length` (Unicode-aware, inclusive bounds), `is_numeric` (ASCII digits only), `alphabetic`/`alphanumeric` (Unicode-aware), `no_whitespace`, and `parsable::<T>` (parses via `T: FromStr` — needs the turbofish, since `T` doesn't appear in `Validator`'s return type). `parsable` isn't limited to numbers: any `FromStr` type works, including ones from other crates — `parsable::<chrono::NaiveDate>(..)` gives a correct, leap-year-aware date validator without `ratiform` depending on `chrono`.

All the shape-based validators pass on an empty string (they check *shape*, not presence — that's the required check's job); `parsable` doesn't, since `"".parse()` fails like any other malformed input.

For anything these don't cover, `.validator(...)` still takes a plain closure — the built-ins are a convenience on top, not a replacement.

## More examples

* [`examples/quickstart.rs`](examples/quickstart.rs) — the Quick start above, complete and runnable.
* [`examples/typed-fields.rs`](examples/typed-fields.rs) — the [Field identifiers](#field-identifiers) walkthrough, complete and runnable.
* [`examples/theming.rs`](examples/theming.rs) — a custom `FormStyle` with distinct colors for labels and values (see [Theming](#theming)), a deliberately long label to show off wrapping, and debug fields exercising `set_value`/`focused_field`.
* [`examples/login-form.rs`](examples/login-form.rs) — a login screen with a masked password, a custom strength check, and the form rendered inside a titled, centered `Block`.
* [`examples/connections.rs`](examples/connections.rs) — `validators::parsable::<u16>` for a port number, a `Select` for the protocol, required and optional fields side by side.
* [`examples/normalizer.rs`](examples/normalizer.rs) — forcing a field to stay uppercase or lowercase via `normalizer(...)`.
* [`examples/text-area.rs`](examples/text-area.rs) — a form built entirely around a `TextArea`.
* [`examples/layouts.rs`](examples/layouts.rs) — `Horizontal` vs `Stacked` [`FormLayout`](#layout), recomputed from the available width every frame so it reacts to a terminal resize on its own.
* [`examples/custom.rs`](examples/custom.rs) — `FormLayout::Custom` grid layout (see [Custom layout](#custom-layout)), two fields side by side, a field spanning the full row, and a three-column address group.

Run any of them with `cargo run --example <n>`.

## Theming

By default, `Form::default()` renders with a built-in gray/bold/reversed scheme. Build a [`ratiform::style::FormStyle`](src/style.rs) and hand it to `Form::with_style(...)` instead:

```rust
use ratatui::style::{Color, Style};
use ratiform::style::{FieldStyle, FormStyle};

let normal = Style::default().fg(Color::LightGreen);
let my_style = FormStyle::builder()
    .value(FieldStyle::builder().normal(normal).focused(normal.bold()).build())
    .error(Style::default().fg(Color::Red).bold())
    .build();

frame.render_stateful_widget(Form::with_style(my_style), area, &mut state);
```

`FormStyle` groups five areas — `label`, `value`, `highlight` (the focused field / selected row), `error`, and `placeholder`. The first three are a [`FieldStyle`](src/style.rs) each (one `Style` per `normal`/`focused`/`disabled`/`readonly` state, resolved for you); `error`/`placeholder` are plain `Style`s. Full field-by-field docs are on `FormStyle`/`FieldStyle` themselves (`cargo doc --open`). A runnable example with a full custom theme is in [`examples/theming.rs`](examples/theming.rs).

## Keyboard navigation

| Key | Action |
| --- | --- |
| `Tab` / `Shift+Tab` | Move to the next / previous field |
| `Ctrl+Enter` | Submit, unless some field is currently invalid |
| `Enter` | Same as `Ctrl+Enter`, unless the focused field claims it (a `TextArea` inserts a newline instead) |
| `Esc` | Cancel the form |

Every other key goes to the focused field — `Space` toggles a checkbox, arrow keys navigate a select or move a text cursor, and so on.

## Form result

```rust
match state.result() {
    ratiform::FormResult::Submitted => { /* ... */ }
    ratiform::FormResult::Cancelled => { /* ... */ }
    ratiform::FormResult::Working => { /* still editing */ }
}
```

A few methods work at any point while the form is active, not just after submission:

* `value(&self, id: &T) -> Option<String>` / `set_value(&mut self, id: &T, value: &str)` — read or overwrite a single field; `set_value` validates immediately, like a keystroke would.
* `value_as::<V: FromStr>(&self, id: &T) -> Option<Result<V, V::Err>>` — parses the current value as any `FromStr` type; pairs naturally with a `parsable::<V>` validator on the same field, but doesn't check for one.
* `focused_field(&self) -> Option<&T>` — the id of the field that currently has focus.
* `is_dirty(&self) -> bool` / `is_field_dirty(&self, id: &T) -> Option<bool>` / `reset(&mut self)` — whether a value has changed since the form was built, and rolling it back.

Once the form is `Submitted`/`Cancelled`, `values(self) -> impl Iterator<Item = (T, String)>` consumes the form and collects every field:

```rust
let values: HashMap<FormField, String> = state.values().collect();
```

Full signatures and edge cases for all of these are in the generated docs.

## Current status

This project is still in an early stage. The core design — typed field identifiers, the builder/state/widget split, validation — is becoming stable, but breaking changes are still possible before a first tagged release.

Contributions, ideas and bug reports are welcome. If you're thinking about adding a new field kind, see [`docs/adding-a-widget.md`](docs/adding-a-widget.md) for the wiring points and conventions the existing four already follow.

## License

`ratiform` is dual-licensed under the [MIT License](LICENSE-MIT) and the [Apache License 2.0](LICENSE-APACHE). You may choose either.

## A note about AI

I used AI to help me write some of the tests and documentation for this project. Writing tests can be a bit tedious, and English isn't my native language, so AI has been a useful tool to speed things up and improve the documentation.

I still review, adapt, and run the generated tests, but I prefer to be transparent about how AI was used in this project.
