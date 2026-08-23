use ratatui::widgets::ListState;

use crate::widget::{
    check_box::CheckBoxStatus, select::SelectStatus, single_line::SingleLineStatus,
};

pub enum Requirement {
    Required,
    Optional,
}

pub struct FieldOptions {
    pub(crate) required: Requirement,
    pub(crate) disabled: bool,
    pub(crate) readonly: bool,
    pub(crate) height: u16,
}
impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            required: Requirement::Required,
            disabled: false,
            readonly: false,
            height: 1,
        }
    }
}

pub struct Field {
    pub(crate) kind: FieldKind,
    pub(crate) options: FieldOptions,
}

impl Field {
    pub fn label(&self) -> &str {
        self.kind.label()
    }
    pub fn value(&self) -> String {
        self.kind.value()
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
    pub fn value(&self) -> String {
        match self {
            FieldKind::SingleLine(k) => k.value.clone(),
            FieldKind::CheckBox(k) => k.checked.to_string(),
            FieldKind::Select(k) => k.values.first().map_or("".to_string(), |f| f.0.clone()),
        }
    }
}
