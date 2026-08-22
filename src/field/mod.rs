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

impl SingleLine {
    fn byte_position(&self, position: u16, default: usize) -> usize {
        self.value
            .char_indices()
            .nth(position as usize)
            .map_or(default, |(i, _)| i)
    }
    pub(crate) fn delete(&mut self) {
        if self.value.is_empty() {
            return;
        }
        let byte_idx = self.byte_position(self.position, self.value.len());
        if byte_idx < self.value.len() {
            self.value.remove(byte_idx);
            self.position = self.position.min(self.value.chars().count() as u16)
        }
    }
    pub(crate) fn backspace(&mut self) {
        if self.position == 0 {
            return;
        }
        let byte_idx = self.byte_position(self.position - 1, 0);
        self.value.remove(byte_idx);
        self.position = self.position.saturating_sub(1)
    }
    pub(crate) fn left(&mut self) {
        self.position = self.position.saturating_sub(1)
    }
    pub(crate) fn right(&mut self) {
        self.position = (self.position + 1).min(self.value.chars().count() as u16)
    }
    pub(crate) fn home(&mut self) {
        self.position = 0
    }
    pub(crate) fn end(&mut self) {
        self.position = self.value.len() as u16
    }
    pub(crate) fn insert(&mut self, c: char) {
        let byte_idx = self.byte_position(self.position, self.value.len());
        self.value.insert(byte_idx, c);
        self.position += 1;
    }
}

pub struct CheckBox {
    pub(crate) label: String,
    pub(crate) checked: bool,
}

impl CheckBox {
    pub(crate) fn toggle(&mut self) {
        self.checked = !self.checked
    }
}

pub struct Select {
    pub(crate) label: String,
    pub(crate) values: Vec<String>,
}
