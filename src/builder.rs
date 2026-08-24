use ratatui::widgets::ListState;

use crate::{
    FormState,
    field::{Field, FieldKind, FieldOptions, Requirement},
    widget::{
        check_box::{CheckBoxStatus, CheckboxBuilder},
        select::{SelectBuilder, SelectStatus},
        single_line::{SingleLineBuilder, SingleLineStatus},
    },
};

pub struct FormBuilder<T> {
    pub(crate) fields: Vec<Field<T>>,
}

impl<T> Default for FormBuilder<T> {
    fn default() -> Self {
        Self {
            fields: Default::default(),
        }
    }
}

impl<T: PartialEq> FormBuilder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn single_line(self, id: T, label: impl Into<String>) -> SingleLineBuilder<T> {
        SingleLineBuilder {
            id,
            form: self,
            label: label.into(),
            value: String::new(),
            options: FieldOptions::default(),
            masked_with: None,
            placeholder: None,
        }
    }

    pub fn checkbox(self, id: T, label: impl Into<String>) -> CheckboxBuilder<T> {
        CheckboxBuilder {
            id,
            form: self,
            label: label.into(),
            checked: false,
            options: FieldOptions::default(),
        }
    }

    pub fn select(self, id: T, label: impl Into<String>) -> SelectBuilder<T> {
        SelectBuilder {
            id,
            form: self,
            label: label.into(),
            values: Vec::new(),
            options: FieldOptions::default(),
            selected: 0,
        }
    }

    pub fn build(self) -> FormState<T> {
        FormState::new(self.fields)
    }
}

// MACRO
#[macro_export]
macro_rules! field_builder_common {
    ($builder:ident<$generic:ident>) => {
        impl<$generic: PartialEq> $builder<$generic> {
            // FieldOptions
            pub fn required(mut self) -> Self {
                self.options.required = $crate::field::Requirement::Required;
                self
            }
            pub fn optional(mut self) -> Self {
                self.options.required = $crate::field::Requirement::Optional;
                self
            }
            pub fn disabled(mut self) -> Self {
                self.options.disabled = true;
                self
            }
            pub fn readonly(mut self) -> Self {
                self.options.readonly = true;
                self
            }
            pub fn height(mut self, height: u16) -> Self {
                self.options.height = height;
                self
            }

            pub fn validator<F>(mut self, function: F) -> Self
            where
                F: Fn(&str) -> Result<(), String> + 'static,
            {
                self.options.validator.push(Box::new(function));
                self
            }

            // Builders
            pub fn single_line(
                self,
                id: $generic,
                label: impl Into<String>,
            ) -> $crate::widget::single_line::SingleLineBuilder<$generic> {
                self.finish().single_line(id, label)
            }
            pub fn checkbox(
                self,
                id: $generic,
                label: impl Into<String>,
            ) -> $crate::widget::check_box::CheckboxBuilder<$generic> {
                self.finish().checkbox(id, label)
            }
            pub fn select(
                self,
                id: $generic,
                label: impl Into<String>,
            ) -> $crate::widget::select::SelectBuilder<$generic> {
                self.finish().select(id, label)
            }

            //
            pub fn build(self) -> FormState<T> {
                self.finish().build()
            }
        }
    };
}
