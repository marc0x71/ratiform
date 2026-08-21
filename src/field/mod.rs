pub enum Requirement {
    Required,
    Optional,
}

pub struct FieldOptions {
    pub(crate) required: Requirement,
    pub(crate) disabled: bool,
    pub(crate) readonly: bool,
}
impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            required: Requirement::Required,
            disabled: false,
            readonly: false,
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
    SingleLine(SingleLine),
    CheckBox(CheckBox),
    Select(Select),
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
            FieldKind::Select(k) => k.values.first().map_or("".to_string(), |f| f.clone()),
        }
    }
}

pub struct SingleLine {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) position: u16,
}

pub struct CheckBox {
    pub(crate) label: String,
    pub(crate) checked: bool,
}

pub struct Select {
    pub(crate) label: String,
    pub(crate) values: Vec<String>,
}
