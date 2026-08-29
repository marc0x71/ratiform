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
        if let Some(row) = self.current_row.take()
            && !row.is_empty()
        {
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

#[cfg(test)]
mod builder_tests {
    use super::*;
    use ratatui::layout::Constraint;

    #[test]
    fn consecutive_row_calls_without_cells_do_not_create_a_phantom_row() {
        let built = CustomLayoutBuilder::new()
            .row()
            .row()
            .label(Constraint::Fill(1), 1)
            .build();

        let expected = CustomLayout::new(vec![vec![(
            Constraint::Fill(1),
            Some(Object::new(ObjectKind::Label, 1)),
        )]]);
        assert_eq!(built, expected);
    }

    #[test]
    fn an_intentional_all_none_spacer_row_is_kept() {
        let built = CustomLayoutBuilder::<i32>::new()
            .row()
            .empty(Constraint::Fill(1))
            .build();
        assert_eq!(
            built,
            CustomLayout::new(vec![vec![(Constraint::Fill(1), None)]])
        );
    }
}
