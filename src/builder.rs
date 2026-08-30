use crate::{
    FormState,
    error::BuildError,
    field::{Field, FieldOptions},
    widget::{
        check_box::CheckboxBuilder, select::SelectBuilder, single_line::SingleLineBuilder,
        text_area::TextAreaBuilder,
    },
};

/// Entry point for building a form. Generic over `T`, the type used to
/// identify each field — any type works, from a plain integer to your own
/// `enum`; see the crate-level docs for why that matters.
pub struct FormBuilder<T> {
    // TODO: pending_error records the first build error found while
    // chaining field builders, deferred until build() is called -- a
    // fluent chain always runs every call regardless of earlier
    // failures, so this records the first problem instead of stopping
    // the chain. Review if a fallible-per-field API is ever worth the
    // ergonomics trade-off.
    pub(crate) fields: Vec<Field<T>>,
    pub(crate) label_width: Option<u16>,
    pub(crate) pending_error: Option<BuildError>,
}

impl<T> Default for FormBuilder<T> {
    fn default() -> Self {
        Self {
            fields: Default::default(),
            label_width: None,
            pending_error: None,
        }
    }
}

impl<T: PartialEq> FormBuilder<T> {
    /// Starts a new, empty form.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fixes the label column at `width` characters (plus one for
    /// breathing room between label and value), instead of letting it be
    /// computed automatically from the widest label. Unlike the automatic
    /// calculation — which caps itself to a third of the available area —
    /// an explicit `label_width` is not capped: if you set it, you're
    /// asking for exactly that, regardless of how much space is actually
    /// available.
    pub fn label_width(mut self, width: u16) -> FormBuilder<T> {
        self.label_width = Some(width);
        self
    }

    /// Adds a single-line text field.
    pub fn single_line(self, id: T, label: impl Into<String>) -> SingleLineBuilder<T> {
        SingleLineBuilder {
            id,
            form: self,
            label: label.into(),
            value: String::new(),
            options: FieldOptions::default(),
            masked_with: None,
            placeholder: None,
            alphabet: None,
        }
    }

    /// Adds a checkbox field.
    pub fn checkbox(self, id: T, label: impl Into<String>) -> CheckboxBuilder<T> {
        CheckboxBuilder {
            id,
            form: self,
            label: label.into(),
            checked: false,
            options: FieldOptions::default(),
        }
    }

    /// Adds a select field: a list of `(value, label)` pairs the user picks
    /// from with the arrow keys. `value` is what `values()`/`value()`
    /// return once selected; `label` is what's shown on screen.
    pub fn select(self, id: T, label: impl Into<String>) -> SelectBuilder<T> {
        SelectBuilder {
            id,
            form: self,
            label: label.into(),
            values: Vec::new(),
            options: FieldOptions::default(),
            selected: Some(0),
        }
    }

    /// Adds a text-area field.
    pub fn text_area(self, id: T, label: impl Into<String>) -> TextAreaBuilder<T> {
        TextAreaBuilder {
            id,
            form: self,
            label: label.into(),
            value: String::new(),
            options: FieldOptions::default(),
            placeholder: None,
        }
    }

    /// Builds the `FormState`. Every field is validated once immediately,
    /// so a field that starts out invalid (an initial value too short, a
    /// required field left empty) already carries its error before the
    /// first render.
    pub fn build(self) -> Result<FormState<T>, BuildError> {
        if let Some(err) = self.pending_error {
            return Err(err);
        }
        Ok(FormState::new(self.fields, self.label_width))
    }

    pub(crate) fn push_field(&mut self, field: Field<T>) {
        if self.pending_error.is_none()
            && let Some(pos) = self.fields.iter().position(|f| f.id == field.id)
        {
            self.pending_error = Some(BuildError::DuplicateFieldId {
                first: pos,
                duplicate: self.fields.len(),
            });
        }
        self.fields.push(field);
    }
}

/// Generates the builder methods shared by every field kind: `required`,
/// `optional`, `disabled`, `readonly`, `height`, `validator`, plus the
/// chaining methods that let you add another field or call `build()`
/// without breaking out of the current builder chain.
///
/// Invoked once per widget builder (see `single_line.rs`, `check_box.rs`,
/// `select.rs`), so a new field kind gets all of this for free instead of
/// reimplementing it.
macro_rules! field_builder_common {
    ($builder:ident<$generic:ident>) => {
        impl<$generic: PartialEq> $builder<$generic> {
            /// Marks the field as required, using `message` as the error
            /// shown when it's left empty. Every field is required by
            /// default already, with a built-in message — call this only
            /// when you want your own message instead. Calling it replaces
            /// whatever required state the field had before, including a
            /// prior `optional()`.
            pub fn required(mut self, message: String) -> Self {
                self.options.required = Some($crate::validators::required(message));
                self
            }

            /// Opts the field out of the required check entirely: an empty
            /// value is valid, and no validators run on it.
            pub fn optional(mut self) -> Self {
                self.options.required = None;
                self
            }

            /// Disables the field: it stops responding to keyboard input,
            /// and `Tab`/`BackTab` skip over it entirely — it can never
            /// receive focus.
            ///
            /// This only affects *interactive* input. A disabled field
            /// still responds to programmatic changes —
            /// [`FormState::set_value`](crate::FormState::set_value) and
            /// [`FormState::reset`](crate::FormState::reset) work on it
            /// exactly as on any other field — and it still participates in
            /// validation: a `required` field left empty still blocks
            /// submission even while disabled, and since `Tab` can't reach
            /// it, the user has no way to fix that from the keyboard.
            pub fn disabled(mut self) -> Self {
                self.options.disabled = true;
                self
            }

            /// Makes the field readonly: like `disabled()`, it stops
            /// responding to keyboard input and the same caveats about
            /// [`FormState::set_value`](crate::FormState::set_value),
            /// [`FormState::reset`](crate::FormState::reset), and
            /// validation apply. Unlike `disabled()`, though, a readonly
            /// field is **not** skipped by `Tab`/`BackTab` — it can still
            /// receive focus, it just won't accept edits once it does.
            pub fn readonly(mut self) -> Self {
                self.options.readonly = true;
                self
            }

            /// Sets the field's height in terminal rows, not counting the
            /// row(s) reserved for its error message (added automatically,
            /// growing to fit a message that wraps onto more than one
            /// line). Defaults to 1; mainly useful for `Select`, to
            /// control how many options are visible without scrolling.
            pub fn height(mut self, height: u16) -> Self {
                self.options.height = height;
                self
            }

            /// Adds a validation rule, run whenever the field's value is
            /// non-empty (an empty value is handled by the required check
            /// instead, see `required`/`optional` above). Can be called
            /// more than once on the same field: each call adds one more
            /// check, run in the order they were added, and the field is
            /// invalid as soon as one of them returns `Err`.
            pub fn validator<F>(mut self, function: F) -> Self
            where
                F: Fn(&str) -> Result<(), String> + Send + 'static,
            {
                self.options.validator.push(Box::new(function));
                self
            }

            /// Rewrites the field's value into a canonical form every time it
            /// changes — a keystroke, `set_value`, or the initial value
            /// given at build time — before validation runs on it.
            /// Typical use: forcing a fiscal code to uppercase, or an
            /// email address to lowercase.
            ///
            /// Only one `normalizer` is kept per field; calling this again
            /// replaces the previous one, unlike `validator(...)`, which
            /// accumulates. Where `validator(...)` judges an already-typed
            /// value, `normalizer` rewrites it — pair it with
            /// [`alphabet`](crate::widget::single_line::SingleLineBuilder::alphabet)
            /// on a `SingleLine` field if you also want to reject certain
            /// characters outright rather than rewrite them.
            pub fn normalizer<F>(mut self, function: F) -> Self
            where
                F: Fn(&str) -> String + Send + 'static,
            {
                self.options.normalizer = Some(Box::new(function));
                self
            }

            // Builders
            /// Finishes this field and starts a new single-line text field,
            /// continuing the same builder chain.
            pub fn single_line(
                self,
                id: $generic,
                label: impl Into<String>,
            ) -> $crate::widget::single_line::SingleLineBuilder<$generic> {
                self.finish().single_line(id, label)
            }

            /// Finishes this field and starts a new checkbox field,
            /// continuing the same builder chain.
            pub fn checkbox(
                self,
                id: $generic,
                label: impl Into<String>,
            ) -> $crate::widget::check_box::CheckboxBuilder<$generic> {
                self.finish().checkbox(id, label)
            }

            /// Finishes this field and starts a new select field,
            /// continuing the same builder chain.
            pub fn select(
                self,
                id: $generic,
                label: impl Into<String>,
            ) -> $crate::widget::select::SelectBuilder<$generic> {
                self.finish().select(id, label)
            }

            /// Finishes this field and starts a new multi-line text field,
            /// continuing the same builder chain.
            pub fn text_area(
                self,
                id: $generic,
                label: impl Into<String>,
            ) -> $crate::widget::text_area::TextAreaBuilder<$generic> {
                self.finish().text_area(id, label)
            }

            /// Finishes this field and sets the form's label column width — see
            /// `FormBuilder::label_width`.
            pub fn label_width(self, width: u16) -> FormBuilder<T> {
                self.finish().label_width(width)
            }

            /// Finishes this field and builds the `FormState` — see
            /// `FormBuilder::build`.
            pub fn build(self) -> Result<FormState<T>, $crate::error::BuildError> {
                self.finish().build()
            }
        }
    };
}
pub(crate) use field_builder_common;
