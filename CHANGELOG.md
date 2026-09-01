# Changelog

All notable changes to `ratiform` are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Cargo's SemVer rules](https://doc.rust-lang.org/cargo/reference/semver.html)
for `0.x` releases (a breaking change bumps the minor version, not the patch).

## [Unreleased]

### Added
- `FormState::reset_field(&id)` — resets a single field to its initial
  value, instead of the whole form.
- `FormState::dirty_fields()` — an iterator over the ids of fields whose
  value has changed since the form was built.
- `FormState::has_errors()` and `FormState::errors()` — whether any field
  is currently invalid, and an iterator over the ids and messages of the
  ones that are.
- `FormState::commit()` — moves every field's current value to become its
  new baseline, without rebuilding the form. Useful after loading an
  existing record for editing (`set_value()` doesn't move the baseline on
  its own) or after a successful save. Revalidates in the process, so
  `error` stays accurate even if the committed value isn't valid.

## [0.5.3] - 2026-09-01

### Added
- Field visibility: `hide()`/`show()` on any field builder set the starting
  state; `FormState::set_visible(&id, bool)` toggles it at runtime,
  `is_field_visible(&id)` reads it back. A hidden field draws nothing —
  label, value, and error alike — in every layout, and is skipped by
  `Tab`/`Shift+Tab`, same as `disabled()`. `Horizontal`/`Stacked` reclaim
  its space automatically; `Custom` draws its cells as empty rather than
  removing them from the grid, so surrounding cells don't reflow — build
  a different `CustomLayout` for that state if you want the space back.
  Unlike `disabled()`/`readonly()`, a hidden field is also excluded from
  validation, so a required-but-empty field doesn't block submission while
  hidden.

- `FormState::field(&id) -> Option<FieldRef<'_>>` and its per-kind
  shortcuts — `single_line()`, `select()`, `checkbox()`, `text_area()` —
  read-only, widget-specific views into a field's state, for information
  `value()`/`value_as()` can't express as a `String`:
  - `SingleLineRef`: `value()`, `cursor_position()`.
  - `SelectRef`: `selected_index()`, `selected_value()`, `selected_label()`
    — `selected_index()` in particular has no `String`-based equivalent.
  - `CheckBoxRef`: `checked()`.
  - `TextAreaRef`: `value()`, `cursor_position()`, `lines()`,
    `line_count()`, `scroll_offset()` (meaningful only after the field has
    been rendered at least once).

  Each accessor returns `None` if the field doesn't exist or isn't of that
  kind — unlike the rest of the crate's typed-id guarantees, there's no
  compile-time link between a field's id and its widget kind.

### Fixed
- `FormState::reset()` now also resets `result` back to `Working`, `focus`
  to the first field, and clears `cursor_position` — previously it only
  restored field values, leaving a `Submitted`/`Cancelled` form with no way
  to become usable again short of rebuilding it from scratch.

### Documentation
- Added `examples/todo-list.rs`, a small in-memory todo app demonstrating
  the form as a transient overlay for adding/editing a task rather than
  the whole screen: `reset()` before opening it empty, `set_value()` to
  pre-fill it for editing, and `value_as::<Priority>()` parsing a `Select`
  into a custom enum via `FromStr`.

- Documented `FieldRef` in the README ("Reading widget-specific state",
  "Choosing between `value()`, `value_as()`, and `FieldRef`"), including
  the three equivalent ways `examples/todo-list.rs` reads its `Priority`
  field.

## [0.5.2] - 2026-08-31

### Added
- Configurable symbols for `Select` and `Checkbox`:
  - `SelectBuilder::highlight_symbol(...)` — symbol shown before the selected row (defaults to `"> "`).
  - `CheckboxBuilder::symbols(...)` — checked/unchecked symbols (defaults to `"[✓]"` / `"[ ]"`).

### Fixed
- Fixed public field builder documentation linking to the private
  `field_builder_common!` macro.
- `Ctrl`/`Alt` held with a character no longer inserts it into `SingleLine`,
  `TextArea`, or toggles `Checkbox` (`Ctrl+Space`) — lets an application
  treat those combinations as global shortcuts without the keystroke also
  leaking into the focused field. `Shift` is unaffected, since `Shift+letter`
  is normal typing (uppercase, symbols), not a shortcut.

### Changed
- `field_builder_common!` is no longer `#[macro_export]`-ed. Technically a
  public item removal, but the macro referenced `pub(crate)` fields of
  `FieldOptions` internally, so it could never compile when invoked from
  outside the crate — not a real capability lost.

### Chore
- Declared `rust-version = "1.88.0"` in `Cargo.toml` (found with
  `cargo-msrv`). The floor isn't edition 2024 alone (`1.85`) — it's the
  let-chains used throughout the codebase, stabilized specifically in
  `1.88`.

## [0.5.1] - 2026-08-30

### Fixed
- Long validation error messages now wrap across multiple lines instead of
  being truncated, in all three layouts (`Horizontal`, `Stacked`, `Custom`).
  The row reserved for an error grows to fit the current message, with a
  floor of one line even when the field currently has no error (so the
  layout doesn't jump the moment one appears).
- `stacked.rs`: a field squeezed by the surrounding scroll could render only
  its label, with neither value nor error shown, even when space remained
  for the value — fixed by making label/value/error independently
  scrollable rows instead of one all-or-nothing block per field.
- `horizontal.rs`: the label's wrap height was computed against the full row
  width instead of the label column's actual width, understating how much
  space a wrapped label needed.

### Documentation
- Noted in the README that a `CustomLayout` cell referencing an unknown
  field id silently draws nothing, rather than erroring.
- Updated the README and the `.height()` doc comment to reflect that error
  messages can now span more than one line.

## [0.5.0] - 2026-08-30

### Added
- `FormBuilder::build()` (and every field builder's `.build()`) now returns
  `Result<FormState<T>, BuildError>` instead of `FormState<T>` directly —
  **breaking change**. Fails if two fields share the same id, or if a
  `Select` field has two options with the same value; both were previously
  silent (the second field/value was unreachable through any accessor, with
  no error).
- `SelectBuilder::no_selection()` — starts a `Select` with nothing chosen.
  The only way `.required()` has any effect on a `Select`; previously the
  default first-option selection made it trivially always satisfied.
- `CheckboxBuilder::must_be_checked()` — a validator scoped to `Checkbox`
  only, for cases like a terms-and-conditions box where `.required()` alone
  never fails (an unchecked box is still the non-empty string `"false"`).

### Fixed
- `Ctrl+Enter` on an invalid form no longer leaks into a focused `TextArea`
  as a plain `Enter` (inserting a newline instead of the submit being
  silently rejected).
- A `readonly` `TextArea` no longer silently swallows `Enter` — `readonly`/
  `disabled` fields no longer claim special keys they can't act on.

### Documentation
- `set_value()`, `disabled()`, `readonly()`, `Select::selected()`: cargodoc
  now documents behavior that was already correct but previously unstated
  (per-kind value coercion, the `disabled`/`readonly` interaction with
  validation and focus cycling, out-of-range index clamping).
- README: new "Duplicate ids and values" section explaining when and why
  `build()` fails; a note on the `disabled()` + `required()` interaction in
  the "Validation" section; a "Tip" on `CustomLayout` about giving every
  field a `Value` cell.

[0.5.1]: https://github.com/marc0x71/ratiform/releases/tag/v0.5.1
[0.5.0]: https://github.com/marc0x71/ratiform/releases/tag/v0.5.0
