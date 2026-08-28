use ratatui::layout::Constraint;

use crate::layout::custom::{CustomLayout, Object, ObjectKind};

// BUILDER
#[derive(Debug)]
pub struct CustomLayoutBuilder<T> {
    rows: Vec<Vec<(Constraint, Option<Object<T>>)>>,
    current_row: Option<Vec<(Constraint, Option<Object<T>>)>>,
}

impl<T> Default for CustomLayoutBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CustomLayoutBuilder<T> {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            current_row: None,
        }
    }

    pub fn row(mut self) -> Self {
        self.push_current_row();
        self.current_row = Some(Vec::new());
        self
    }

    pub fn label(self, constraint: Constraint, id: T) -> Self {
        self.cell(constraint, (ObjectKind::Label, id))
    }

    pub fn value(self, constraint: Constraint, id: T) -> Self {
        self.cell(constraint, (ObjectKind::Value, id))
    }

    pub fn error(self, constraint: Constraint, id: T) -> Self {
        self.cell(constraint, (ObjectKind::Error, id))
    }

    pub fn cell(mut self, constraint: Constraint, object: impl Into<Object<T>>) -> Self {
        self.current_row
            .get_or_insert_with(Vec::new)
            .push((constraint, Some(object.into())));

        self
    }

    pub fn empty(mut self, constraint: Constraint) -> Self {
        self.current_row
            .get_or_insert_with(Vec::new)
            .push((constraint, None));

        self
    }

    pub fn end_row(mut self) -> Self {
        self.push_current_row();
        self
    }

    pub fn build(mut self) -> CustomLayout<T> {
        self.push_current_row();

        CustomLayout::new(self.rows)
    }

    fn push_current_row(&mut self) {
        if let Some(row) = self.current_row.take() {
            self.rows.push(row);
        }
    }
}

#[macro_export]
macro_rules! custom_layout {
    (
        $(
            row [
                $(
                    ($constraint:expr, $object:ident $($id:tt)*)
                ),* $(,)?
            ]
        ),* $(,)?
    ) => {{
        let mut rows = Vec::new();
        $(
            let mut row = Vec::new();
            $(
                row.push((
                    $constraint,
                    custom_layout!(
                        @object
                        $object $($id)*
                    ),
                ));
            )*
            rows.push(row);
        )*
        $crate::layout::custom::CustomLayout::new(rows)
    }};
    (@object None) => {
        None
    };
    (@object Label($id:expr)) => {
        Some($crate::layout::custom::Object::new($crate::layout::custom::ObjectKind::Label, $id))
    };
    (@object Value($id:expr)) => {
        Some($crate::layout::custom::Object::new($crate::layout::custom::ObjectKind::Value, $id))
    };
    (@object Error($id:expr)) => {
        Some($crate::layout::custom::Object::new($crate::layout::custom::ObjectKind::Error, $id))
    };
}
