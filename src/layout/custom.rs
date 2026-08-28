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

#[derive(Debug, Clone, PartialEq)]
pub struct CustomLayout<T> {
    pub(crate) rows: Vec<Vec<(Constraint, Option<Object<T>>)>>,
}
impl<T> Default for CustomLayout<T> {
    fn default() -> Self {
        Self { rows: Vec::new() }
    }
}
impl<T> CustomLayout<T> {
    pub fn new(rows: Vec<Vec<(Constraint, Option<Object<T>>)>>) -> Self {
        Self { rows }
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
