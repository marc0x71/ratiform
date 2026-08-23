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

#[derive(Default)]
pub struct FormBuilder {
    pub(crate) fields: Vec<Field>,
}

impl FormBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn single_line(self, label: impl Into<String>) -> SingleLineBuilder {
        SingleLineBuilder {
            form: self,
            label: label.into(),
            value: String::new(),
            options: FieldOptions::default(),
        }
    }

    pub fn checkbox(self, label: impl Into<String>) -> CheckboxBuilder {
        CheckboxBuilder {
            form: self,
            label: label.into(),
            checked: false,
            options: FieldOptions::default(),
        }
    }

    pub fn select(self, label: impl Into<String>) -> SelectBuilder {
        SelectBuilder {
            form: self,
            label: label.into(),
            values: Vec::new(),
            options: FieldOptions::default(),
            selected: 0,
        }
    }

    pub fn build(self) -> FormState {
        FormState::new(self.fields)
    }
}

// MACRO
#[macro_export]
macro_rules! field_builder_common {
    ($builder:ty) => {
        impl $builder {
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

            // Builders
            pub fn single_line(
                self,
                label: impl Into<String>,
            ) -> $crate::widget::single_line::SingleLineBuilder {
                self.finish().single_line(label)
            }
            pub fn checkbox(
                self,
                label: impl Into<String>,
            ) -> $crate::widget::check_box::CheckboxBuilder {
                self.finish().checkbox(label)
            }
            pub fn select(self, label: impl Into<String>) -> $crate::widget::select::SelectBuilder {
                self.finish().select(label)
            }

            //
            pub fn build(self) -> FormState {
                self.finish().build()
            }
        }
    };
}
