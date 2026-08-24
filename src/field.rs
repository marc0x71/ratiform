use crate::{
    style::FieldState,
    validators,
    widget::{check_box::CheckBoxStatus, select::SelectStatus, single_line::SingleLineStatus},
};

pub type Validator = Box<dyn Fn(&str) -> Result<(), String> + 'static>;

pub struct FieldOptions {
    pub(crate) required: Option<Validator>, // None = optional
    pub(crate) disabled: bool,
    pub(crate) readonly: bool,
    pub(crate) height: u16,
    pub(crate) validator: Vec<Validator>,
}

impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            required: Some(validators::required("<*>".to_string())),
            disabled: false,
            readonly: false,
            height: 1,
            validator: Vec::new(),
        }
    }
}
impl FieldOptions {
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

pub struct Field<T> {
    pub(crate) id: T,
    pub(crate) kind: FieldKind,
    pub(crate) options: FieldOptions,
    pub(crate) error: Option<String>,
    pub(crate) initial_value: String,
}

impl<T> Field<T> {
    pub fn label(&self) -> &str {
        self.kind.label()
    }

    pub fn get(&self) -> String {
        self.kind.get()
    }

    pub(crate) fn set(&mut self, value: &str) {
        self.kind.set(value);
        self.validate();
    }

    pub(crate) fn validate(&mut self) {
        let value = self.kind.get();
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

    pub fn is_dirty(&self) -> bool {
        self.kind.get() != self.initial_value
    }

    pub(crate) fn reset(&mut self) {
        self.kind.set(&self.initial_value);
        self.validate();
    }
}

pub enum FieldKind {
    SingleLine(SingleLineStatus),
    CheckBox(CheckBoxStatus),
    Select(SelectStatus),
}

impl FieldKind {
    pub fn label(&self) -> &str {
        match self {
            FieldKind::SingleLine(k) => k.label.as_str(),
            FieldKind::CheckBox(k) => k.label.as_str(),
            FieldKind::Select(k) => k.label.as_str(),
        }
    }
    pub fn get(&self) -> String {
        match self {
            FieldKind::SingleLine(k) => k.get(),
            FieldKind::CheckBox(k) => k.get(),
            FieldKind::Select(k) => k.get(),
        }
    }
    pub fn set(&mut self, value: &str) {
        match self {
            FieldKind::SingleLine(k) => k.set(value),
            FieldKind::CheckBox(k) => k.set(value),
            FieldKind::Select(k) => k.set(value),
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
            }),
            options: FieldOptions {
                required,
                disabled: false,
                readonly: false,
                height: 1,
                validator,
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
