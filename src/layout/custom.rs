use ratatui::layout::Constraint;

use crate::layout::builder::CustomLayoutBuilder;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKind {
    Label,
    Value,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object<T> {
    pub(crate) kind: ObjectKind,
    pub(crate) id: T,
}

impl<T> Object<T> {
    pub fn new(kind: ObjectKind, id: T) -> Self {
        Self { kind, id }
    }
}

const DEFAULT_COLUMN_GAP: u16 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct CustomLayout<T> {
    pub(crate) rows: Vec<Vec<(Constraint, Option<Object<T>>)>>,
    pub(crate) column_gap: u16,
}
impl<T> Default for CustomLayout<T> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            column_gap: DEFAULT_COLUMN_GAP,
        }
    }
}
impl<T> CustomLayout<T> {
    pub fn new(rows: Vec<Vec<(Constraint, Option<Object<T>>)>>) -> Self {
        Self {
            rows,
            column_gap: DEFAULT_COLUMN_GAP,
        }
    }

    pub fn with_column_gap(mut self, gap: u16) -> Self {
        self.column_gap = gap;
        self
    }

    pub fn builder() -> CustomLayoutBuilder<T> {
        CustomLayoutBuilder::new()
    }
}

impl<T> From<(ObjectKind, T)> for Object<T> {
    fn from((kind, id): (ObjectKind, T)) -> Self {
        Self { kind, id }
    }
}
