use ratatui::widgets::ListState;

use crate::{
    FormState,
    field::{Field, FieldKind, FieldOptions, Requirement},
    widget::{check_box::CheckBoxStatus, select::SelectStatus, single_line::SingleLineStatus},
};

#[derive(Default)]
pub struct FormBuilder {
    fields: Vec<Field>,
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

macro_rules! field_builder_common {
    ($builder:ty) => {
        impl $builder {
            // FieldOptions
            pub fn required(mut self) -> Self {
                self.options.required = Requirement::Required;
                self
            }
            pub fn optional(mut self) -> Self {
                self.options.required = Requirement::Optional;
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
            pub fn single_line(self, label: impl Into<String>) -> SingleLineBuilder {
                self.finish().single_line(label)
            }
            pub fn checkbox(self, label: impl Into<String>) -> CheckboxBuilder {
                self.finish().checkbox(label)
            }
            pub fn select(self, label: impl Into<String>) -> SelectBuilder {
                self.finish().select(label)
            }

            //
            pub fn build(self) -> FormState {
                self.finish().build()
            }
        }
    };
}

// SINGLE LINE
pub struct SingleLineBuilder {
    form: FormBuilder,
    label: String,
    value: String,
    options: FieldOptions,
}

impl SingleLineBuilder {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    fn finish(mut self) -> FormBuilder {
        let position = self.value.len() as u16;
        self.form.fields.push(Field {
            kind: FieldKind::SingleLine(SingleLineStatus {
                label: self.label,
                value: self.value,
                position,
            }),
            options: self.options,
        });

        self.form
    }
}
field_builder_common!(SingleLineBuilder);

// CHECK BOX
pub struct CheckboxBuilder {
    form: FormBuilder,
    label: String,
    checked: bool,
    options: FieldOptions,
}

impl CheckboxBuilder {
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    fn finish(mut self) -> FormBuilder {
        self.form.fields.push(Field {
            kind: FieldKind::CheckBox(CheckBoxStatus {
                label: self.label,
                checked: self.checked,
            }),
            options: self.options,
        });

        self.form
    }
}
field_builder_common!(CheckboxBuilder);

pub struct SelectBuilder {
    form: FormBuilder,
    label: String,
    values: Vec<(String, String)>,
    selected: usize,
    options: FieldOptions,
}

impl SelectBuilder {
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    pub fn values_ref(mut self, input: &[(&str, &str)]) -> Self {
        self.values = input
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();

        self
    }

    pub fn values<I, K, V>(mut self, input: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.values = input
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();

        self
    }

    fn finish(mut self) -> FormBuilder {
        self.form.fields.push(Field {
            kind: FieldKind::Select(SelectStatus {
                label: self.label,
                values: self.values,
                list_state: ListState::default().with_selected(Some(self.selected)),
            }),
            options: self.options,
        });

        self.form
    }
}
field_builder_common!(SelectBuilder);
