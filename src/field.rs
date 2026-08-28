use std::borrow::Cow;

use ratatui::crossterm::event::KeyCode;

use crate::{
    style::FieldState,
    validators,
    widget::{
        check_box::CheckBoxStatus, select::SelectStatus, single_line::SingleLineStatus,
        text_area::TextAreaStatus,
    },
};

/// A per-field validation rule: `Ok(())` if `value` is acceptable, `Err`
/// with the message to show otherwise. Built by
/// [`crate::validators`], or written by hand and passed to
/// `.validator(...)` on any field builder.
pub type Validator = Box<dyn Fn(&str) -> Result<(), String> + Send + 'static>;

/// A per-field rewrite rule, run before validation: takes the current
/// value, returns the value it should become (e.g. forcing uppercase).
/// Passed to `.normalizer(...)` on any field builder.
pub type Normalizer = Box<dyn Fn(&str) -> String + Send + 'static>;

/// The options shared by every field kind, populated by
/// [`field_builder_common`](crate::field_builder_common) — see that macro
/// for what each one does.
pub struct FieldOptions {
    pub(crate) required: Option<Validator>, // None = optional
    pub(crate) disabled: bool,
    pub(crate) readonly: bool,
    pub(crate) height: u16,
    pub(crate) validator: Vec<Validator>,
    pub(crate) normalizer: Option<Normalizer>, // None = optional
}

impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            required: Some(validators::required("<*>".to_string())),
            disabled: false,
            readonly: false,
            height: 1,
            validator: Vec::new(),
            normalizer: None,
        }
    }
}
impl FieldOptions {
    /// Resolves which of a [`crate::style::FieldStyle`]'s four states
    /// applies right now: `disabled` beats `readonly`, which beats
    /// `focused` (`has_focus`), which beats `normal`.
    pub(crate) fn to_field_state(&self, has_focus: bool) -> FieldState {
        if self.disabled {
            FieldState::Disabled
        } else if self.readonly {
            FieldState::Readonly
        } else if has_focus {
            FieldState::Focused
        } else {
            FieldState::Normal
        }
    }
}

/// One field in a form: its id, its kind-specific state, the options
/// shared across kinds, its current validation error (if any), and the
/// value it had when the form was built (for `is_dirty`/`reset`).
pub struct Field<T> {
    pub(crate) id: T,
    pub(crate) kind: FieldKind,
    pub(crate) options: FieldOptions,
    pub(crate) error: Option<String>,
    pub(crate) initial_value: String,
}

impl<T> Field<T> {
    /// The field's label, as given to `.single_line(id, label)` and
    /// friends.
    pub fn label(&self) -> &str {
        self.kind.label()
    }

    /// The field's current value, as an owned `String`. Prefer
    /// [`Field::get_ref`] internally when a borrow is enough — this
    /// allocates every time, needed at the public API boundary
    /// (`FormState::value`/`values`) but wasteful on a hot path like a
    /// keystroke.
    pub fn get(&self) -> String {
        self.kind.get()
    }

    /// Same as [`Field::get`], but borrows the value instead of cloning it
    /// when the underlying kind already stores it as owned text
    /// (`SingleLine`/`Select`/`TextArea`) — at moment only `Checkbox` still
    /// allocates, since a `bool` has no string representation to borrow.
    /// Used by `normalize`/`validate`/`is_dirty`, which only ever need to
    /// read the value, never to own it.
    pub fn get_ref(&self) -> Cow<'_, str> {
        self.kind.get_ref()
    }

    pub(crate) fn set(&mut self, value: &str) {
        self.kind.set(value);
        self.normalize();
        self.validate();
    }

    /// Rewrites the value with `options.normalizer`, if one is set — see
    /// [`Normalizer`]. A no-op otherwise. Called before
    /// [`Field::validate`] from every path that can change a field's
    /// value: a keystroke, `set_value`, and once at build time.
    pub(crate) fn normalize(&mut self) {
        if let Some(function) = self.options.normalizer.as_ref() {
            let value = self.get_ref();
            self.kind.set(function(&value).as_str());
        }
    }

    /// Recomputes `error` from `options.required`/`options.validator`
    /// against the current value. See the crate README's "Validation"
    /// section for the exact required/validator/empty-value interaction
    /// this implements — this is the one place it's enforced.
    pub(crate) fn validate(&mut self) {
        let value = self.get_ref();
        let required_error = self.options.required.as_ref().and_then(|f| f(&value).err());
        self.error = if let Some(error) = required_error {
            Some(error)
        } else {
            if !value.is_empty() {
                self.options.validator.iter().find_map(|f| f(&value).err())
            } else {
                None
            }
        }
    }

    pub(crate) fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Whether the current value differs from `initial_value` — the value
    /// the field had right after the form was built.
    pub fn is_dirty(&self) -> bool {
        self.kind.get_ref() != self.initial_value
    }

    pub(crate) fn reset(&mut self) {
        self.kind.set(&self.initial_value);
        self.validate();
    }

    /// Key codes this field wants to handle itself even when they'd
    /// otherwise be reserved globally by [`crate::FormState::handle_input`]
    /// — today, only `TextArea` claims `Enter`, to insert a newline
    /// instead of submitting the form.
    pub fn special_key_handled(&self) -> Vec<KeyCode> {
        self.kind.special_key_handled()
    }
}

/// The per-kind state behind a [`Field`] — add a variant here (and to
/// every method below) to introduce a new field kind. See
/// `docs/adding-a-widget.md` for the full checklist.
pub enum FieldKind {
    SingleLine(SingleLineStatus),
    CheckBox(CheckBoxStatus),
    Select(SelectStatus),
    TextArea(TextAreaStatus),
}

impl FieldKind {
    pub fn label(&self) -> &str {
        match self {
            FieldKind::SingleLine(k) => k.label.as_str(),
            FieldKind::CheckBox(k) => k.label.as_str(),
            FieldKind::Select(k) => k.label.as_str(),
            FieldKind::TextArea(k) => k.label.as_str(),
        }
    }

    /// Owned form of the current value — see each `XxxStatus::get`'s own
    /// notes for the string format it uses (e.g. `Checkbox`'s
    /// `"true"`/`"false"`, `Select`'s value-not-label convention).
    pub fn get(&self) -> String {
        match self {
            FieldKind::SingleLine(k) => k.get(),
            FieldKind::CheckBox(k) => k.get(),
            FieldKind::Select(k) => k.get(),
            FieldKind::TextArea(k) => k.get(),
        }
    }

    /// Borrowed form of [`FieldKind::get`] — see [`Field::get_ref`] for
    /// why this exists.
    pub fn get_ref(&self) -> Cow<'_, str> {
        match self {
            FieldKind::SingleLine(k) => k.get_ref(),
            FieldKind::CheckBox(k) => k.get_ref(),
            FieldKind::Select(k) => k.get_ref(),
            FieldKind::TextArea(k) => k.get_ref(),
        }
    }

    pub fn set(&mut self, value: &str) {
        match self {
            FieldKind::SingleLine(k) => k.set(value),
            FieldKind::CheckBox(k) => k.set(value),
            FieldKind::Select(k) => k.set(value),
            FieldKind::TextArea(k) => k.set(value),
        }
    }

    pub fn special_key_handled(&self) -> Vec<KeyCode> {
        match self {
            FieldKind::TextArea(_) => vec![KeyCode::Enter],
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod validate_tests {
    use super::*;
    use crate::widget::single_line::SingleLineStatus;

    fn make_field(
        value: &str,
        required: Option<Validator>,
        validator: Vec<Validator>,
    ) -> Field<i32> {
        Field {
            id: 1,
            kind: FieldKind::SingleLine(SingleLineStatus {
                label: "Test".to_owned(),
                value: value.to_owned(),
                position: 0,
                masked_with: None,
                placeholder: None,
                alphabet: None,
            }),
            options: FieldOptions {
                required,
                disabled: false,
                readonly: false,
                height: 1,
                validator,
                normalizer: None,
            },
            error: None,
            initial_value: value.to_owned(),
        }
    }

    fn always_ok() -> Validator {
        Box::new(|_: &str| Ok(()))
    }

    fn always_fails(message: &'static str) -> Validator {
        Box::new(move |_: &str| Err(message.to_owned()))
    }

    #[test]
    fn required_and_empty_is_invalid_with_required_message() {
        let mut field = make_field(
            "",
            Some(validators::required("Campo obbligatorio".to_owned())),
            vec![],
        );
        field.validate();
        assert_eq!(field.error, Some("Campo obbligatorio".to_owned()));
    }

    #[test]
    fn required_and_non_empty_with_no_validators_is_valid() {
        let mut field = make_field(
            "Mario",
            Some(validators::required("Campo obbligatorio".to_owned())),
            vec![],
        );
        field.validate();
        assert_eq!(field.error, None);
    }

    #[test]
    fn optional_and_empty_is_valid() {
        // This is the exact regression case for the bug fixed earlier: an
        // optional field, left empty, with no validators, must be valid.
        let mut field = make_field("", None, vec![]);
        field.validate();
        assert_eq!(field.error, None);
    }

    #[test]
    fn optional_and_empty_skips_validators_too() {
        // Even a validator that would always fail must not run on an
        // optional, empty field: an empty value never reaches the Vec.
        let mut field = make_field("", None, vec![always_fails("should not run")]);
        field.validate();
        assert_eq!(field.error, None);
    }

    #[test]
    fn required_and_non_empty_runs_validators() {
        let mut field = make_field(
            "ab",
            Some(validators::required("Campo obbligatorio".to_owned())),
            vec![validators::min_length(3, "Too short".to_owned())],
        );
        field.validate();
        assert_eq!(field.error, Some("Too short".to_owned()));
    }

    #[test]
    fn first_failing_validator_wins_the_message() {
        let mut field = make_field(
            "x",
            Some(validators::required("Campo obbligatorio".to_owned())),
            vec![always_ok(), always_fails("B"), always_fails("C")],
        );
        field.validate();
        assert_eq!(field.error, Some("B".to_owned()));
    }

    #[test]
    fn all_validators_passing_is_valid() {
        let mut field = make_field(
            "x",
            Some(validators::required("Campo obbligatorio".to_owned())),
            vec![always_ok(), always_ok()],
        );
        field.validate();
        assert_eq!(field.error, None);
    }

    #[test]
    fn required_error_short_circuits_before_validators_run() {
        // This validator panics if it is ever called. If this test passes
        // without panicking, we've proven the validators are never invoked
        // on an empty field, not just that their result gets discarded.
        let panicking_validator: Validator =
            Box::new(|_: &str| panic!("this validator should not have been called"));

        let mut field = make_field(
            "",
            Some(validators::required("Obbligatorio".to_owned())),
            vec![panicking_validator],
        );
        field.validate();
        assert_eq!(field.error, Some("Obbligatorio".to_owned()));
    }

    #[test]
    fn set_updates_value_and_revalidates() {
        let mut field = make_field(
            "Mario",
            Some(validators::required("Campo obbligatorio".to_owned())),
            vec![],
        );
        field.validate();
        assert_eq!(field.error, None);

        field.set("");
        assert_eq!(field.error, Some("Campo obbligatorio".to_owned()));
    }

    #[test]
    fn reset_restores_initial_value_and_revalidates() {
        let mut field = make_field(
            "Mario",
            Some(validators::required("Campo obbligatorio".to_owned())),
            vec![],
        );
        field.validate();
        assert_eq!(field.error, None);

        field.set("");
        assert_eq!(field.error, Some("Campo obbligatorio".to_owned()));

        field.reset();
        assert_eq!(field.get(), "Mario");
        assert_eq!(field.error, None);
    }

    #[test]
    fn is_dirty_is_false_when_value_matches_initial() {
        let field = make_field("Mario", None, vec![]);
        assert!(!field.is_dirty());
    }

    #[test]
    fn is_dirty_is_true_after_the_value_changes() {
        let mut field = make_field("Mario", None, vec![]);
        field.set("Luigi");
        assert!(field.is_dirty());
    }
}
