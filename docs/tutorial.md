# ratiform tutorial

A hands-on, feature-by-feature guide to `ratiform`. If you just want the
five-minute version, see the [Quick start](../README.md#quick-start) in the
README instead — this document goes deeper into each piece, one chapter per
feature, and can be read in any order once you're past the first two
chapters.

## Table of contents

1. [Installation](#1-installation)
2. [Anatomy of a form](#2-anatomy-of-a-form)
3. [Field identifiers](#3-field-identifiers)
4. [Single-line field](#4-single-line-field)
5. [Checkbox](#5-checkbox)
6. [Select](#6-select)
7. [Multi-select](#7-multi-select)
8. [Text area](#8-text-area)
9. [Options shared by every field](#9-options-shared-by-every-field)
10. [Validation](#10-validation)
11. [Normalizing values](#11-normalizing-values)
12. [Reading and driving the form at runtime](#12-reading-and-driving-the-form-at-runtime)
13. [Layout](#13-layout)
14. [Theming](#14-theming)
15. [Keyboard navigation](#15-keyboard-navigation)
16. [Handling build errors](#16-handling-build-errors)
17. [Where to go next](#17-where-to-go-next)

---

## 1. Installation

```sh
cargo add ratiform
```

`ratiform` targets Ratatui `0.30` and Rust `1.88` or later. To track the
unreleased development version instead:

```toml
[dependencies]
ratiform = { git = "https://github.com/marc0x71/ratiform" }
```

## 2. Anatomy of a form

Every form in `ratiform` passes through three types:

- **[`FormBuilder`](https://docs.rs/ratiform/latest/ratiform/builder/struct.FormBuilder.html)**
  — a fluent chain that declares each field and its options.
- **`FormState<T>`** — what `build()` produces. Your application owns it for
  as long as the form is on screen: it holds every field's current value,
  which field has focus, and the form's `Submitted` / `Cancelled` / `Working`
  result.
- **`Form<T>`** — a stateless `StatefulWidget`. Build a fresh one every
  frame (it's cheap — just a style and a layout) and render it against your
  `FormState`, exactly like any other Ratatui widget.

```rust
use ratiform::{Form, builder::FormBuilder};

#[derive(Debug, PartialEq)]
enum Field {
    Username,
    Password,
}

// 1. Build: declare fields, get a Result back.
let mut state = FormBuilder::new()
    .single_line(Field::Username, "Username")
    .single_line(Field::Password, "Password")
    .masked()
    .build()?;

// 2. Render: every frame, in your draw closure.
// frame.render_stateful_widget(Form::default(), area, &mut state);

// 3. Drive: feed key events as they arrive.
// state.handle_input(key_event);

// 4. React: check the result once input has been handled.
// match state.result() {
//     ratiform::FormResult::Submitted => { /* read state.values() */ }
//     ratiform::FormResult::Cancelled => { /* ... */ }
//     ratiform::FormResult::Working => { /* keep looping */ }
// }
```

Note that `build()` returns `Result<FormState<T>, BuildError>`, not a bare
`FormState<T>` — see [chapter 16](#16-handling-build-errors) for when it can
fail and how to handle it. A complete, runnable version of the loop above is
in [`examples/quickstart.rs`](../examples/quickstart.rs).

## 3. Field identifiers

The first argument to `single_line()`, `checkbox()`, `select()`,
`multi_select()` and `text_area()` is the field's **id** — the value you'll
later use to read that field back out of the form. `FormBuilder`,
`FormState` and `Form` are all generic over its type `T`, which needs
`PartialEq` (and `Hash + Eq` too, if you want to `collect()` results into a
`HashMap`). It doesn't have to be a string or an integer — an `enum` is the
usual, and best, choice:

```rust
use ratiform::builder::FormBuilder;

#[derive(Debug, Hash, Eq, PartialEq)]
enum Field {
    FirstName,
    LastName,
    Country,
}

let state = FormBuilder::new()
    .single_line(Field::FirstName, "First name")
    .single_line(Field::LastName, "Last name")
    .select(Field::Country, "Country")
    .values_ref(&[("IT", "Italy"), ("FR", "France")])
    .build()?;

let value = state.value(&Field::FirstName);
```

`state.value(&Field::Email)` — not `state.value("email")`. The compiler
catches a typo'd field name at compile time, and a `match` on every field's
result warns you if you forget a variant. There is no string key anywhere,
and no serialization round-trip to a form-specific data model. This is the
one design decision most of the rest of the crate is built around; see the
[project README](../README.md#why-ratiform) if you want the full story of
why it's there. A complete example living entirely on this idea is
[`examples/typed-fields.rs`](../examples/typed-fields.rs).

## 4. Single-line field

`.single_line(id, label)` is a single row of editable text: insertion,
`Backspace`/`Delete`, and cursor movement with `Left`/`Right`/`Home`/`End`.

```rust
FormBuilder::new()
    .single_line(Field::Name, "Name")
    .value("Mario")
    .placeholder("Enter your name")
    .required("Name is required".to_owned())
    .build()?;
```

Beyond the options every field kind shares (chapter 9), a single-line field
adds:

| Method | What it does |
| --- | --- |
| `.value(v)` | Sets the initial value. |
| `.placeholder(p)` | Text shown, dimmed, while the value is empty. |
| `.masked()` / `.masked_with(c)` | Draws every character as `•` (or `c`) instead of what was typed — purely cosmetic: validation and `value()` still see the real text. |
| `.alphabet(chars)` | Rejects any keystroke for a character not in `chars`, outright — the character is never inserted. |

`alphabet` and [`normalizer`](#11-normalizing-values) solve related but
different problems: `alphabet` refuses a character before it ever reaches
the field, `normalizer` accepts a character and rewrites it afterwards
(e.g. forcing it to uppercase). Use `alphabet` to keep certain characters
out entirely, `normalizer` to canonicalize what's typed.

## 5. Checkbox

`.checkbox(id, label)` is a boolean, toggled with `Space`:

```rust
FormBuilder::new()
    .checkbox(Field::Terms, "I accept the terms")
    .checked(false)
    .must_be_checked("You must accept the terms to continue".to_owned())
    .build()?;
```

| Method | What it does |
| --- | --- |
| `.checked(b)` | Sets the initial state. Defaults to `false`. |
| `.must_be_checked(message)` | The field is invalid, with `message`, while unchecked — the checkbox equivalent of `.required(...)` (which, on a checkbox, would only reject an *empty* value, and a checkbox's value is never empty). |
| `.symbols(checked, unchecked)` | Overrides the two glyphs drawn for the checked/unchecked states (`"[✓]"`/`"[ ]"` by default). |

`state.value(&id)` on a checkbox returns `"true"` or `"false"`. To read it as
an actual `bool`, either use `state.value_as::<bool>(&id)` or, more
directly, `state.checkbox(&id).map(|c| c.checked())` (see
[chapter 12](#12-reading-and-driving-the-form-at-runtime)).

## 6. Select

`.select(id, label)` is a list of `(value, label)` pairs the user picks from
with the arrow keys. `value` is what `value()`/`values()` return once an
option is selected; `label` is what's actually drawn.

```rust
FormBuilder::new()
    .select(Field::Country, "Country")
    .values_ref(&[("IT", "Italy"), ("FR", "France"), ("DE", "Germany")])
    .selected(1)
    .height(5)
    .build()?;
```

| Method | What it does |
| --- | --- |
| `.values_ref(&[(v, l), ...])` | Sets the options from a literal slice of borrowed strings — the common case. |
| `.values(iter)` | Sets the options from any `IntoIterator` of owned-or-convertible pairs (a `Vec<(String, String)>`, a `HashMap`, ...) — for data built at runtime. |
| `.selected(i)` | Selects option `i` initially. Not bounds-checked at call time; an index out of range once the field's final option list is known is silently clamped to the last option, never a panic. |
| `.no_selection()` | Starts with nothing selected. Mutually exclusive with `.selected(i)` — whichever is called last wins. This is the only way `.required(...)` has any effect on a `Select`: with a selection always present otherwise, the required check could never fail. |
| `.horizontal()` / `.vertical()` | Lays the options out on a single scrolling row navigated with `Left`/`Right` (and `Home`/`End`), instead of the default column navigated with `Up`/`Down`/`Home`/`End`/`PageUp`/`PageDown`. |
| `.searchable()` | Turns the list into a filter-as-you-type combobox — see below. |
| `.scrollbar(bool)` | Shows a scrollbar reflecting the current position: vertical on the right edge when the field is `.vertical()` (the list loses one column of width to make room), horizontal along the bottom edge when it's `.horizontal()` (the field needs one extra row). |
| `.highlight_symbol(s)` | The marker drawn before the highlighted option (`"> "` by default). Ignored in `horizontal()` mode, where the cursor is shown via style alone. |
| `.spacing(n)` / `.preview(n)` | Horizontal-mode-only: spaces between options, and how many options past the selected one stay visible while scrolling. Both default to `2`. |

### Searchable select

`.searchable()` turns the field into a filter-as-you-type combobox: typed
characters narrow the options down to those whose label matches as a
fuzzy, case-insensitive subsequence, and the matched characters are drawn
with the `MATCH` style part (see [chapter 14](#14-theming)). `Backspace`
removes the last character of the query rather than editing a value;
`Esc` clears the query first — a second `Esc`, with an empty query,
cancels the form as usual. With no query, every option is shown in its
original order.

```rust
FormBuilder::new()
    .select(Field::Country, "Country")
    .values_ref(&[("IT", "Italy"), ("FR", "France"), ("DE", "Germany")])
    .searchable()
    .height(5)
    .build()?;
```

`FormState::set_value` and `FormState::reset` both clear the search query
first, so a value set programmatically is never masked by a query left
over from a previous interaction.

`state.value(&id)` returns the selected option's *value*, never its label.
To reach both — plus the selected index and the current query — use
`state.select(&id)`, which returns a `SelectRef`:

```rust
let state = FormBuilder::new()
    .select(Field::Country, "Country")
    .values_ref(&[("IT", "Italy"), ("FR", "France")])
    .selected(1)
    .build()?;

let sel = state.select(&Field::Country).unwrap();
assert_eq!(sel.selected_value(), Some("FR"));
assert_eq!(sel.selected_label(), Some("France"));
assert_eq!(sel.selected_index(), Some(1));
```

`sel.search_query()` returns the current query as `Some("")` before any
character is typed, or `None` if `.searchable()` wasn't set at all.
`sel.filtered_count()` returns how many options match that query — handy
for a "3 matches" hint next to the field. It's the total number of options
when the query is empty, or when the field isn't searchable at all.

## 7. Multi-select

`.multi_select(id, label)` is the same idea as `Select`, but any number of
options can be toggled with `Space`. Unlike `Select`, the cursor (which
option `Up`/`Down` moves over) and the selection (which options are
checked) are independent — moving the cursor never changes what's selected.

```rust
FormBuilder::new()
    .multi_select(Field::Tags, "Tags")
    .values_ref(&[
        ("bug", "Bug"),
        ("feature", "Feature"),
        ("docs", "Documentation"),
    ])
    .selected(&[0]) // "bug" checked from the start
    .height(3)
    .optional()
    .build()?;
```

Most methods mirror `Select`: `.values_ref(...)`, `.values(...)`,
`.no_selection()`, `.horizontal()`/`.vertical()`, `.scrollbar(bool)`,
`.spacing(n)`, `.preview(n)`. Two differences worth calling out:

- `.selected(indices)` takes a **slice** of indices, not a single one — it
  checks every option listed.
- `.symbols(selected, unselected)` sets the two glyphs used for a checked
  and unchecked option (`"✓"`/`" "` by default) — the multi-select
  equivalent of `Checkbox::symbols`.

### Pinning and searching

`.pinnable()` keeps every checked option pinned at the top of the list, in
front of the unchecked ones, so a long list never hides what's already
selected. The list re-pins itself after every `Space`, and the cursor
moves down to the next row afterwards, so checking several options in a
row doesn't keep jumping back to the top.

`.searchable()` works the same as [`Select`'s](#searchable-select), with
one difference worth knowing: checking/unchecking never depends on the
query, so a checked option that no longer matches the typed text would
normally vanish from the list along with everything else that doesn't
match. Combine it with `.pinnable()` if you want a checked option to stay
visible even while it doesn't match the current query.

```rust
FormBuilder::new()
    .multi_select(Field::Country, "Countries")
    .values_ref(&[("IT", "Italy"), ("FR", "France"), ("DE", "Germany")])
    .pinnable()
    .searchable()
    .height(5)
    .build()?;
```

`state.multi_select(&id).unwrap().search_query()` reads the current query
back, mirroring `SelectRef::search_query()`. [`examples/multi-select.rs`](../examples/multi-select.rs)
combines both on its `Tags` field.

`state.multi_select(&id).unwrap().search_query()` reads the current query
back, mirroring `SelectRef::search_query()`. [`examples/multi-select.rs`](../examples/multi-select.rs)
combines both on its `Tags` field.

`state.value(&id)` returns every selected option's value joined into one
comma-separated `String` (e.g. `"bug,docs"`) — handy for storing as a single
field, but usually not what you want to iterate over. `state.multi_select(&id)`
returns a `MultiSelectRef` with the values already split apart:

```rust
// state built as above, with e.g. "bug" and "docs" both selected
let tags: Vec<&str> = state
    .multi_select(&Field::Tags)
    .map(|r| r.selected_values().collect())
    .unwrap_or_default();
```

`selected_labels()` gives the on-screen text instead of the values, and
`selected()` gives the raw selected indices. Because values are joined with
`,`, an option whose own value contains a comma is rejected at `build()`
time — see [`InvalidMultiSelectValue`](#16-handling-build-errors).

`selected_index()` is the odd one out: it's the option under the cursor,
not part of the selection, and always an index into the original list of
options — even while the list is filtered by a query or reordered by
`.pinnable()`.

A complete program built around this field kind is
[`examples/multi-select.rs`](../examples/multi-select.rs).

## 8. Text area

`.text_area(id, label)` is multi-line text, with the same
insertion/deletion/placeholder support as a single-line field, plus
`Up`/`Down`, scrolling, and `PageUp`/`PageDown`. `Home`/`End` jump to the
start/end of the current visual line (`End` stops on the last character of a
wrapped row); `Ctrl+Home`/`Ctrl+End` jump to the start/end of the whole text.
Long lines wrap at the character level, not at word boundaries — the same
default `vim` uses — and width is measured in terminal cells, so wide
characters take two columns and are never split.

```rust
FormBuilder::new()
    .text_area(Field::Notes, "Notes")
    .placeholder("Write here...")
    .scrollbar(true)
    .height(5)
    .build()?;
```

| Method | What it does |
| --- | --- |
| `.value(v)` | Sets the initial text. |
| `.placeholder(p)` | Text shown while the value is empty. |
| `.scrollbar(bool)` | Shows a scrollbar when the text overflows `.height(n)`. |

Since `Enter` inserts a newline instead of submitting the form, submitting
while a `TextArea` has focus needs `Ctrl+Enter` — see
[Keyboard navigation](#15-keyboard-navigation). `state.text_area(&id)`
returns a `TextAreaRef` with `value()`, `cursor_position()` (`(column, row)`),
`index_position()`, `lines()` (the text split into its visual lines) and
`line_count()`, on top of the usual `state.value(&id)`. A form built entirely
around this field kind is [`examples/text-area.rs`](../examples/text-area.rs).

## 9. Options shared by every field

These are available on every field builder, right after `single_line()`,
`checkbox()`, `select()`, `multi_select()` or `text_area()`, and chain the
same way regardless of which kind you're configuring:

| Method | What it does |
| --- | --- |
| `.required(message)` | Every field is required by default, with a built-in message — call this only to supply your own. |
| `.optional()` | Opts the field out of the required check: an empty value is valid, and no validators run on it. |
| `.disabled()` | Stops the field from receiving keyboard input at all; `Tab`/`BackTab` skip over it — it can never gain focus. |
| `.readonly()` | Also stops keyboard input, but — unlike `.disabled()` — the field can still receive focus via `Tab`/`BackTab`; it just won't accept edits once it does. |
| `.hide()` / `.show()` | Starts the field hidden, or visible (the default). The runtime equivalent, for toggling later, is `FormState::set_visible` — see below. |
| `.height(n)` | The field's height in rows, not counting the row(s) reserved for its error message. Defaults to `1`; mainly useful to show more than one option of a `Select`/`MultiSelect`, or more than one line of a `TextArea`, without scrolling. |
| `.validator(f)` | Adds one validation rule; see [chapter 10](#10-validation). Can be called more than once. |
| `.normalizer(f)` | Rewrites the value into a canonical form before validation runs; see [chapter 11](#11-normalizing-values). |

A `disabled()` or `readonly()` field is not frozen from your application's
point of view: `FormState::set_value` and `FormState::reset` still work on
it exactly as on any other field, and it still participates in validation —
a `required` field left empty still blocks submission even while
`disabled()`, and since `Tab` can't reach it, the user has no way to fix
that themselves. Because of that, call `.optional()` on any field you
disable without also giving it an initial value, or it's left permanently
invalid *and* permanently unreachable.

A **hidden** field (`.hide()`, or `FormState::set_visible(&id, false)` at
runtime) is a stronger tool than `disabled()`/`readonly()`: it draws
nothing at all — label, value, and error alike — is skipped by
`Tab`/`Shift+Tab`, and, unlike `disabled()`/`readonly()`, is also excluded
from validation, so a required-but-empty field doesn't block submission
while hidden. In `Horizontal`/`Stacked` its space is reclaimed
automatically; in a `Custom` layout its cells draw as empty instead, so
surrounding cells don't reflow — see [chapter 13](#13-layout).
[`examples/visibility.rs`](../examples/visibility.rs) shows `disabled()`,
`readonly()`, and `set_visible` side by side.

## 10. Validation

A field is invalid in exactly two ways:

- **Being required** — every field is required by default; an empty
  required field is invalid, with a built-in or custom (`.required(message)`)
  message.
- **A `.validator(...)` failing** — `Fn(&str) -> Result<(), String>`, run
  any number of times on the same field, in the order it was added.

| Field state | Result |
| --- | --- |
| required, value empty | invalid, required message — validators don't run |
| `.optional()`, value empty | valid — validators don't run |
| any value, non-empty | validators run in order; first `Err` wins, otherwise valid |

An empty value is never handed to a validator — the required check decides
on its own whether an empty value is acceptable, so a validator never needs
to special-case the empty string itself.

Validation runs on every keystroke, on `set_value`, and once up front when
the form is built — a field that starts out invalid already shows its error
on the very first render, with no key needed to trigger it. While any field
is invalid, `Enter`/`Ctrl+Enter` won't submit the form.

```rust
FormBuilder::new()
    .single_line(Field::Name, "Name")
    .validator(validators::min_length(2, "Too short".to_owned()))
    .validator(|v: &str| {
        if v.chars().next().is_some_and(char::is_uppercase) {
            Ok(())
        } else {
            Err("Must start with a capital letter".to_owned())
        }
    })
    .required("Name is required".to_owned())
    .build()?;
```

### Built-in validators

`ratiform::validators` ships the common checks, each taking the error
message and returning a ready-to-use validator:

| Validator | Rejects |
| --- | --- |
| `required(msg)` | An empty value. This is what `.required(msg)` uses internally — you'll rarely call it yourself. |
| `min_length(n, msg)` / `max_length(n, msg)` | A value shorter/longer than `n` characters (Unicode-aware, inclusive bounds). |
| `is_numeric(msg)` | Anything that isn't all ASCII digits — no sign, no decimal point (`"-5"` and `"3.14"` are both rejected). |
| `alphabetic(msg)` / `alphanumeric(msg)` | Anything that isn't all letters / letters-and-digits (Unicode-aware: `à` counts as a letter). |
| `no_whitespace(msg)` | A value containing any whitespace character. |
| `parsable::<T>(msg)` | A value that doesn't parse via `T: FromStr`. Needs the turbofish, since `T` doesn't otherwise appear in the return type: `validators::parsable::<i32>("Not a number".to_owned())`. |

All the shape-based validators above pass on an empty string — they check
*shape*, not presence, which is the required check's job. `parsable` is the
one exception: `"".parse()` fails like any other malformed input, so an
empty value is rejected there too. `parsable` isn't limited to numbers —
`parsable::<chrono::NaiveDate>(...)`, for instance, gives a correct,
leap-year-aware date validator without `ratiform` itself depending on
`chrono`. For anything these don't cover, `.validator(...)` still takes a
plain closure — the built-ins are a convenience on top, not a replacement.
See [`examples/connections.rs`](../examples/connections.rs) for
`parsable::<u16>` validating a port number.

## 11. Normalizing values

Where `.validator(...)` judges an already-typed value, `.normalizer(...)`
rewrites it into a canonical form *before* validation sees it:

```rust
FormBuilder::new()
    .single_line(Field::CodiceFiscale, "Codice fiscale")
    .normalizer(|value: &str| value.to_uppercase())
    .validator(validators::max_length(16, "Too long".to_owned()))
    .build()?;
```

It runs on every keystroke, on `set_value`, and on the field's initial
value — the field's value is never seen, by validators, `is_dirty()`, or
`values()`, in anything other than its normalized form. Only one normalizer
is kept per field; calling `.normalizer(...)` again replaces the previous
one, unlike `.validator(...)`, which accumulates. On a `SingleLine` field it
pairs naturally with [`.alphabet(...)`](#4-single-line-field): `alphabet`
rejects a character outright, `normalizer` rewrites one that was let
through. [`examples/normalizer.rs`](../examples/normalizer.rs) forces a
field to stay uppercase or lowercase as it's typed.

## 12. Reading and driving the form at runtime

A handful of methods on `FormState` work at any point while the form is
active, not only once it's `Submitted`/`Cancelled`.

**Reading a value:**

- `value(&self, id: &T) -> Option<String>` — the field's current value as a
  plain string, or `None` if no field has that id.
- `value_as::<V: FromStr>(&self, id: &T) -> Option<Result<V, V::Err>>` —
  parses the current value as any `FromStr` type. Pairs naturally with a
  `parsable::<V>` validator on the same field, but doesn't check for one, or
  that the `V` here matches the one you validated with.
- `field(&self, id: &T) -> Option<FieldRef<'_>>` — a read-only view whose
  concrete kind isn't known until you downcast it (`to_singleline()`,
  `to_select()`, `to_checkbox()`, `to_textarea()`, `to_multiselect()`, each
  `None` if the field is a different kind).
- `single_line(&self, id)`, `select(&self, id)`, `multi_select(&self, id)`,
  `checkbox(&self, id)`, `text_area(&self, id)` — the more direct route:
  each combines the lookup and the downcast in one call, and is `None` if
  either fails. This is what chapters 6 and 7 used to reach
  `selected_value()`/`selected_values()` beyond the plain `value()` string.

**Writing a value:**

- `set_value(&mut self, id: &T, value: &str)` — overwrites a field's value,
  validating immediately as if the user had typed it. Interpreted
  differently per field kind (a `Checkbox` parses `value`
  case-insensitively as a boolean; a `Select` matches it against the
  field's list of values, and *clears* the selection on no match, rather
  than leaving the previous one in place); never rejected outright.
- `reset(&mut self)` — restores every field to the value it had when the
  form was built, re-validates, moves focus back to the first field, and
  resets `result()` to `Working`. A `Submitted`/`Cancelled` form becomes
  usable again instead of a dead end.
- `reset_field(&mut self, id: &T)` — the same, for a single field.
- `commit(&mut self)` — moves every field's *current* value to become its
  new baseline, so `is_dirty()`/`is_field_dirty()` report `false` again
  until something changes further. Useful after loading an existing record
  into the form for editing (`set_value` alone doesn't move the baseline),
  or after a successful save, without rebuilding the form from scratch.
- `set_visible(&mut self, id: &T, visible: bool)` — shows or hides a field.
  A hidden field draws nothing at all (label, value, error) and is skipped
  by `Tab`/`BackTab`. See [`examples/visibility.rs`](../examples/visibility.rs).

**Inspecting state:**

- `result(&self) -> FormResult` — `Submitted` / `Cancelled` / `Working`.
- `has_errors(&self) -> bool` and `errors(&self) -> impl Iterator<Item = (&T, &str)>`
  — whether any field is currently invalid, and which ones with what
  message.
- `focused_field(&self) -> Option<&T>` — the id of the field with focus
  (`None` only when the form has no fields).
- `is_dirty(&self) -> bool`, `is_field_dirty(&self, id: &T) -> Option<bool>`,
  `dirty_fields(&self) -> impl Iterator<Item = &T>` — whether anything (or
  a specific field) has changed since the form was built or last
  `commit()`ted.

**Once the form is finished:**

- `values(self) -> impl Iterator<Item = (T, String)>` — consumes the form
  and returns every field's id paired with its value; the usual last step
  after `Submitted`/`Cancelled`:

  ```rust
  let values: HashMap<Field, String> = state.values().collect();
  ```

### Choosing between `value()`, `value_as()`, and the `*Ref` accessors

None of the three is strictly "the right one" — they trade off convenience
against how much they can express:

- `value(&id)` — the field's raw `String`, whatever it is. The shortest
  path when a string is all you need.
- `value_as::<V>(&id)` — parses that string via `V: FromStr`. Pairs
  naturally with a `parsable::<V>` validator on the same field.
- `single_line(&id)` / `select(&id)` / etc. — reach for these when you
  need something a plain string genuinely can't express (a cursor
  position, a selected index, a scroll offset), or when you need more
  than one piece of information about the same field at once, since each
  does a single lookup no matter how many of its own methods you then
  call on the result — cheaper than calling `value()` and a `*Ref`
  accessor separately.

[`examples/todo-list.rs`](../examples/todo-list.rs) reads the same
`Select` field all three ways side by side, which is worth a look if the
trade-off isn't obvious yet.

## 13. Layout

By default every field's label and value sit side by side on the same row
— `FormLayout::Horizontal`. Two alternatives are available, set with
`Form::with_layout(...)`:

```rust
Form::default().with_layout(FormLayout::Stacked)
```

- **`FormLayout::Stacked`** puts the label above the value instead, each on
  its own row — useful when the terminal is too narrow for a comfortable
  shared label column:

  ```text
  Horizontal                          Stacked

  Name       [Mario_____________]     Name
  Email      [mario@example.com_]     [Mario_____________]

                                      Email
                                      [mario@example.com_]
  ```

  `Form` is rebuilt fresh every frame, so nothing stops you from switching
  layouts based on the current area — [`examples/layouts.rs`](../examples/layouts.rs)
  does exactly that, reacting to a terminal resize with no extra code
  needed to detect the resize itself.

- **`FormLayout::Custom(CustomLayout<T>)`** is an explicit grid of rows and
  columns, with full control over which cell draws what — a field's label,
  its value, its error, or nothing at all. Build one with the
  `custom_layout!` macro:

  ```rust
  use ratiform::{Form, FormLayout, custom_layout};
  use ratatui::layout::Constraint;

  let layout = custom_layout! {
      row [
          (Constraint::Fill(1), Label(Field::Email)),
          (Constraint::Fill(1), Label(Field::Password)),
      ],
      row [
          (Constraint::Fill(1), Value(Field::Email)),
          (Constraint::Fill(1), Value(Field::Password)),
      ],
      row [
          (Constraint::Length(15), Error(Field::Email)),
          (Constraint::Fill(1), Error(Field::Password)),
      ],
  };
  let form = Form::default().with_layout(FormLayout::Custom(layout));
  ```

  or, equivalently, with `CustomLayoutBuilder`'s fluent chain (`.row()`
  opens a new row; `.label(...)`/`.value(...)`/`.error(...)`/`.empty(...)`
  add cells to it; `.build()` closes whatever's still open). Every field
  cycles through focus regardless of layout, so **give every field a
  `Value` cell somewhere in the grid** — one left out can still receive
  focus, just with no visible cursor and no error to explain why. For the
  same reason, `Tab`/`BackTab` move through fields in the order they were
  *declared*, not the order they appear on screen; lay the grid out to
  match if you want the two to agree.

  Each grid row scrolls independently. In `Horizontal`/`Stacked`, a
  field's label, value, and error always move together as focus changes;
  in a `Custom` layout, only the row that currently has focus is scrolled
  into view, which can leave a distant `Label`/`Error` row scrolled out of
  sight — keep a field's cells on nearby rows if you want them to move
  together. A cell whose id doesn't match any of the form's actual fields
  (a typo, or a layout built separately from the fields it describes)
  draws nothing and reports no error; it's silently treated as an empty
  cell. `CustomLayout::with_column_gap(n)` changes the small horizontal
  gap ratiform inserts between adjacent cells in a row (`1` by default).

  A complete example, including fields spanning several columns and rows,
  is [`examples/custom.rs`](../examples/custom.rs).

`Form::label_width(n)` (on the builder, or `FormState::label_width(&mut self, n)`
at runtime) fixes the shared label column at `n` characters instead of
having it computed automatically from the widest label, capped to a third
of the available area. This only means something under `Horizontal` —
`Stacked` and `Custom` ignore it, since neither has a shared label column.

`ratiform::required_height(&layout, &state, width)` returns the number of
terminal rows a given layout needs to render `state` at a given `width` —
the same figure `Form`'s own rendering computes internally, exposed so you
can size a fixed `Rect` around the form instead of guessing.

## 14. Theming

`Form::default()` renders with a built-in gray/bold/reversed scheme. Hand a
custom `FormStyle` to `Form::with_style(...)` instead:

```rust
use ratatui::style::{Color, Style};
use ratiform::style::{FormStyle, Widgets, Parts, States};

let my_style = FormStyle::builder()
    .add(
        Widgets::ANY,
        Parts::TEXT,
        States::FOCUSED,
        Style::default().fg(Color::LightGreen).bold(),
    )
    .add(
        Widgets::ANY,
        Parts::ERROR,
        States::ANY,
        Style::default().fg(Color::Red).bold(),
    )
    .build();
```

A `FormStyleBuilder` rule is a `(widgets, parts, states, style)` tuple:
`Widgets` (`SINGLE_LINE`, `TEXT_AREA`, `CHECK_BOX`, `SELECT`,
`MULTI_SELECT`, or `ANY` for every kind), `Parts` (`LABEL`, `ERROR`,
`AREA`, `TEXT`, `PLACEHOLDER`, `MARKER`, `ITEM`, `ACTIVE`, `SELECTED`,
`MATCH`, or `ANY` — combine with `|` to target more than one) and `States`
(`NORMAL`, `FOCUSED`, `DISABLED`, `READ_ONLY`, or `ANY` — likewise
combinable with `|`) narrow down exactly what the rule applies to. Rules
can be declared in
any order — `build()` sorts them by specificity and resolves conflicts, so
the only thing declaration order affects is the tie-break between two
equally specific rules, where the one declared last wins. A runnable
example with a full custom theme, distinct colors for labels and values,
and a deliberately long label to show off wrapping, is
[`examples/theming.rs`](../examples/theming.rs). Full field-by-field
reference is in the generated docs (`cargo doc --open`, `Widgets`/`Parts`/
`States`/`FormStyle`).

## 15. Keyboard navigation

A handful of keys are handled globally, regardless of which field has
focus:

| Key | Action |
| --- | --- |
| `Tab` / `Shift+Tab` | Move focus to the next / previous field, wrapping around. |
| `Ctrl+Enter` | Submit, unless some field is currently invalid. |
| `Enter` | Same as `Ctrl+Enter`, unless the focused field claims it for itself (a `TextArea` inserts a newline instead). |
| `Esc` | Cancel the form, unless the focused field claims it for itself (a searchable `Select` clears its search query first — a second `Esc`, with an empty query, cancels as usual). |

Every other key goes to the field with focus — `Space` toggles a
`Checkbox`/an option in a `MultiSelect`, arrow keys navigate a `Select` or
move a text cursor, and so on. A `disabled()` or `readonly()` field simply
drops any key routed to it.

## 16. Handling build errors

`FormBuilder::build()` returns `Result<FormState<T>, BuildError>`. It's an
error, not a panic, because these are mistakes in how the form was
*declared* — bugs to catch in development, not runtime states to design a
UI around — so `?`-ing it in a `main` that returns `Result` is usually all
you need:

```rust
use ratiform::builder::FormBuilder;

fn build_form() -> Result<(), Box<dyn std::error::Error>> {
    let state = FormBuilder::new()
        .single_line(Field::Name, "Name")
        .build()?;
    Ok(())
}
```

`BuildError` has three variants, each pointing at the *positions* of the
offending fields/options (the order they were added in), not their ids —
this keeps `BuildError` itself non-generic:

| Variant | When |
| --- | --- |
| `DuplicateFieldId { first, duplicate }` | Two fields were given the same id. |
| `DuplicateSelectValue { first, duplicate }` | A `Select`/`MultiSelect` has two options with the same value. |
| `InvalidMultiSelectValue { position }` | A `MultiSelect` option's value contains the `,` separator used internally to encode multiple selections into one `String` (see [chapter 7](#7-multi-select)). |

`BuildError` implements `Display` and `std::error::Error`, so it composes
with `anyhow`/`thiserror`-style error handling, or a plain `{err}` in a
format string, without any extra glue.

All three checks run as each field builder finishes, so the error you get
back is always the *first* one found while building the chain — even
though every later field builder in the chain still runs to completion
regardless.

## 17. Where to go next

Every example below is complete and runnable with
`cargo run --example <name>`:

| Example | What it shows |
| --- | --- |
| [`quickstart.rs`](../examples/quickstart.rs) | The [Quick start](../README.md#quick-start) in the README, end to end. |
| [`typed-fields.rs`](../examples/typed-fields.rs) | The [Field identifiers](#3-field-identifiers) walkthrough. |
| [`login-form.rs`](../examples/login-form.rs) | A login screen with a masked password, a custom strength check, and the form inside a titled, centered `Block`. |
| [`connections.rs`](../examples/connections.rs) | `parsable::<u16>` for a port number, a `Select` for the protocol, required and optional fields side by side. |
| [`multi-select.rs`](../examples/multi-select.rs) | A `MultiSelect` for tags plus a horizontal one for permissions. |
| [`text-area.rs`](../examples/text-area.rs) | A form built entirely around a `TextArea`. |
| [`normalizer.rs`](../examples/normalizer.rs) | Forcing a field to stay uppercase or lowercase via `.normalizer(...)`. |
| [`layouts.rs`](../examples/layouts.rs) | `Horizontal` vs `Stacked`, recomputed from the available width on every resize. |
| [`custom.rs`](../examples/custom.rs) | A `Custom` grid layout spanning several fields per row. |
| [`theming.rs`](../examples/theming.rs) | A full custom `FormStyle`, plus `set_value`/`focused_field` exercised from debug fields. |
| [`visibility.rs`](../examples/visibility.rs) | `disabled()`, `readonly()`, and `FormState::set_visible` side by side. |
| [`todo-list.rs`](../examples/todo-list.rs) | A small application using `ratiform` for input rather than a one-shot form. |

For the exact signature and edge cases of any method mentioned here, the
generated API docs are the source of truth:

```sh
cargo doc --open
```

If you're thinking about contributing a new field kind, see
[`docs/adding-a-widget.md`](adding-a-widget.md) for the wiring points and
conventions the existing five already follow.
