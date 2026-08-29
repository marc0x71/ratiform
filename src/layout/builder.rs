use ratatui::layout::Constraint;

use crate::layout::custom::{CustomLayout, Object, ObjectKind};

/// Fluent builder for [`CustomLayout`].
///
/// Cells accumulate into the current row — [`label`](Self::label),
/// [`value`](Self::value), [`error`](Self::error), and
/// [`empty`](Self::empty) all add to whichever row is open. [`row`](Self::row)
/// closes the current row (if any) and opens a new one; [`end_row`](Self::end_row)
/// only closes it, without opening another. [`build`](Self::build) closes
/// whatever's still open and produces the finished [`CustomLayout`].
///
/// ```
/// # use ratiform::layout::builder::CustomLayoutBuilder;
/// # use ratatui::layout::Constraint;
/// # enum Field { Email, Password }
/// let layout = CustomLayoutBuilder::new()
///     .row()
///     .label(Constraint::Length(15), Field::Email)
///     .label(Constraint::Fill(1), Field::Password)
///     .row()
///     .value(Constraint::Length(15), Field::Email)
///     .value(Constraint::Fill(1), Field::Password)
///     .build();
/// ```
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
    /// Creates an empty builder — equivalently, [`CustomLayout::builder`].
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            current_row: None,
        }
    }

    /// Starts a new row — cells added after this belong to it, until the
    /// next `row()`, `end_row()`, or `build()`.
    ///
    /// Calling `row()` (or `end_row()`) again before any cell is added is a
    /// no-op, not an empty row: an unpopulated row is dropped silently
    /// rather than showing up as a zero-cell row in the built layout. For a
    /// visible blank row, add an explicit spacer cell instead — see
    /// [`CustomLayoutBuilder::empty`].
    pub fn row(mut self) -> Self {
        self.push_current_row();
        self.current_row = Some(Vec::new());
        self
    }

    /// Adds a `Label` cell for `id` to the current row. Shorthand for
    /// `cell(constraint, (ObjectKind::Label, id))`.
    pub fn label(self, constraint: Constraint, id: T) -> Self {
        self.cell(constraint, (ObjectKind::Label, id))
    }

    /// Adds a `Value` cell for `id` to the current row — the field's
    /// actual input widget.
    pub fn value(self, constraint: Constraint, id: T) -> Self {
        self.cell(constraint, (ObjectKind::Value, id))
    }

    /// Adds an `Error` cell for `id` to the current row — the field's
    /// validation message, when present. Its row always reserves the
    /// height for it, regardless of whether an error is currently set.
    pub fn error(self, constraint: Constraint, id: T) -> Self {
        self.cell(constraint, (ObjectKind::Error, id))
    }

    /// Adds a cell to the current row from any `T`-keyed object —
    /// [`label`](Self::label), [`value`](Self::value), and
    /// [`error`](Self::error) all delegate to this.
    pub fn cell(mut self, constraint: Constraint, object: impl Into<Object<T>>) -> Self {
        self.current_row
            .get_or_insert_with(Vec::new)
            .push((constraint, Some(object.into())));

        self
    }

    /// Adds a spacer cell — an empty slot in the row, with no label, value,
    /// or error. A row made only of spacer cells is a blank row, useful for
    /// separating groups of fields visually.
    ///
    /// ```
    /// # use ratiform::layout::builder::CustomLayoutBuilder;
    /// # use ratatui::layout::Constraint;
    /// # enum Field { Email, Password }
    /// let layout = CustomLayoutBuilder::new()
    ///     .row()
    ///     .label(Constraint::Fill(1), Field::Email)
    ///     .row()
    ///     .empty(Constraint::Fill(1)) // blank row between the two fields
    ///     .row()
    ///     .label(Constraint::Fill(1), Field::Password)
    ///     .build();
    /// ```
    pub fn empty(mut self, constraint: Constraint) -> Self {
        self.current_row
            .get_or_insert_with(Vec::new)
            .push((constraint, None));

        self
    }

    /// Closes the current row without opening a new one — the next cell
    /// call (if any) starts a fresh row.
    pub fn end_row(mut self) -> Self {
        self.push_current_row();
        self
    }

    /// Finishes the builder: closes the row still being built, if any,
    /// and returns the resulting [`CustomLayout`].
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
