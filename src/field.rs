use ratatui::widgets::ListState;

use crate::{
    style::{FieldState, FieldStyle},
    widget::{check_box::CheckBoxStatus, select::SelectStatus, single_line::SingleLineStatus},
};

pub type Validator = Box<dyn Fn(&str) -> Result<(), String> + 'static>;

pub enum Requirement {
    Required,
    Optional,
}

pub struct FieldOptions {
    pub(crate) required: Requirement,
    pub(crate) disabled: bool,
    pub(crate) readonly: bool,
    pub(crate) height: u16,
    pub(crate) validator: Vec<Validator>,
}

impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            required: Requirement::Required,
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
        self.error = if matches!(self.options.required, Requirement::Required)
            && self.kind.get().is_empty()
        {
            Some("<*>".to_owned())
        } else {
            let value = self.kind.get();
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
