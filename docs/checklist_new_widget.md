# Checklist: new field kind

To check off before committing a new widget (`Xxx`), based on
`check_box.rs` as the simplest existing reference.

## Wiring (the 6 hookup points)

- [ ] **`field.rs`** — new `FieldKind::Xxx(XxxStatus)` variant, plus one
      arm in each of `label()`/`get()`/`set()`.
- [ ] **`event.rs`** — one arm in `handle_input_field` routing to
      `handle_input_xxx`.
- [ ] **`render.rs`** — one arm in the render loop routing to `render_xxx`.
- [ ] **`builder.rs`** — `pub fn xxx(self, id: T, label: impl
      Into<String>) -> XxxBuilder<T>` on `FormBuilder<T>`.
- [ ] **`XxxBuilder<T>`** — invokes `field_builder_common!(XxxBuilder<T>);`
      (don't hand-roll `required`/`optional`/`disabled`/`readonly`/
      `height`/`validator`).
- [ ] **`finish()`** — builds the `Field`, pushes it onto
      `self.form.fields`, sets `initial_value` (needed for
      `is_dirty()`/`reset()`).

## Conventions to follow

- [ ] `get()`/`set()` have an explicit, **documented** string format
      (e.g. `"true"`/`"false"` for checkbox, the key not the label for
      select).
- [ ] `set()` on invalid input **resets**, doesn't ignore — consistent
      with the existing `Checkbox`/`Select`.
- [ ] Every position/length computation uses `.chars().count()`, never
      `.len()` — tested with a string like `"città"`.
- [ ] `render_xxx` returns `Some(position)` only if the widget genuinely
      has a text cursor, otherwise always `None`.
- [ ] `handle_input_xxx` **does not** call `field.validate()` —
      `event::handle_input_field` already does after it returns.

## Tests (only if they clear the filter)

- [ ] Tested what would break silently (a panic on untried input, e.g.
      an unguarded subtraction).
- [ ] Tested what's counterintuitive (e.g. "resets" instead of
      "ignores").
- [ ] **Not** tested: one-line passthroughs to `std`.
- [ ] **Not** tested: anything delegating to Ratatui (e.g. navigation if
      wrapping `ListState`).
- [ ] **Not** tested: rendering on a `Buffer` — no precedent in the
      crate.

## Documentation

- [ ] `XxxBuilder` and its public methods: one line + any non-obvious
      edge case.
- [ ] `XxxStatus`, `handle_input_xxx`, `render_xxx`, the `get()`/`set()`
      format: a shorter doc comment, for whoever implements a fifth
      field kind.
