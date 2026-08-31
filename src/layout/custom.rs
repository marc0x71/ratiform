use ratatui::layout::Constraint;

use crate::layout::builder::CustomLayoutBuilder;

/// What a grid cell in a [`CustomLayout`] draws.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKind {
    /// The field's caption, drawn as wrapped text.
    Label,
    /// The field's actual input widget — the only cell kind the user
    /// interacts with.
    Value,
    /// The field's validation message, if any
    Error,
}

/// A single grid cell's content: what kind of thing to draw
/// ([`ObjectKind`]) and which field it belongs to.
///
/// Built directly with [`Object::new`], or produced by
/// `CustomLayoutBuilder`'s `label`/`value`/`error` methods and
/// the [`custom_layout!`](crate::custom_layout) macro.
#[derive(Debug, Clone, PartialEq)]
pub struct Object<T> {
    pub(crate) kind: ObjectKind,
    pub(crate) id: T,
}

impl<T> Object<T> {
    /// Creates a cell of the given `kind` for the field `id`.
    pub fn new(kind: ObjectKind, id: T) -> Self {
        Self { kind, id }
    }
}

const DEFAULT_COLUMN_GAP: u16 = 1;

/// A form layout laid out as an explicit grid: rows, and inside each
/// row, columns — with full control over which cell (if any) draws
/// what. See [`FormLayout::Custom`](crate::layout::FormLayout::Custom).
///
/// > **Tip:** give every field a `Value` cell somewhere in the grid.
/// > Focus cycles through every field regardless of the layout, so one
/// > left out can still receive focus — just with no visible cursor and
/// > no error to explain why. For the same reason, `Tab`/`BackTab` move
/// > through fields in the order they were declared, not the order they
/// > appear on screen — lay out the grid to match if you want the two to
/// > agree.
///
/// Built with [`CustomLayout::new`] from a raw `Vec` of rows, with
/// [`CustomLayout::builder`]'s fluent API, or with the
/// [`custom_layout!`](crate::custom_layout) macro.
///
/// ```
/// # use ratiform::layout::custom::{CustomLayout, Object, ObjectKind};
/// # use ratatui::layout::Constraint;
/// # enum Field { Name }
/// let layout = CustomLayout::new(vec![
///     vec![(Constraint::Fill(1), Some(Object::new(ObjectKind::Label, Field::Name)))],
///     vec![(Constraint::Fill(1), Some(Object::new(ObjectKind::Value, Field::Name)))],
/// ])
/// .with_column_gap(2);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CustomLayout<T> {
    pub(crate) rows: Vec<Vec<(Constraint, Option<Object<T>>)>>,
    pub(crate) column_gap: u16,
}
impl<T> Default for CustomLayout<T> {
    /// An empty layout (no rows), with the default column gap.
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            column_gap: DEFAULT_COLUMN_GAP,
        }
    }
}
impl<T> CustomLayout<T> {
    /// Builds a layout directly from its rows. Each row is a list of
    /// `(Constraint, cell)` columns — the `Constraint` sizes the column,
    /// the cell is `None` for a spacer or `Some(Object)` for content.
    ///
    /// Prefer [`CustomLayout::builder`] or the
    /// [`custom_layout!`](crate::custom_layout) macro unless you're
    /// assembling rows programmatically.
    pub fn new(rows: Vec<Vec<(Constraint, Option<Object<T>>)>>) -> Self {
        Self {
            rows,
            column_gap: DEFAULT_COLUMN_GAP,
        }
    }

    /// Sets the horizontal gap, in columns, between adjacent cells in
    /// every row — the first column of each row is never padded.
    /// Defaults to 1.
    pub fn with_column_gap(mut self, gap: u16) -> Self {
        self.column_gap = gap;
        self
    }

    /// Starts a `CustomLayoutBuilder` for assembling a layout row by row.
    pub fn builder() -> CustomLayoutBuilder<T> {
        CustomLayoutBuilder::new()
    }
}

/// Shorthand for [`Object::new`] — lets
/// [`CustomLayoutBuilder::cell`](crate::layout::builder::CustomLayoutBuilder::cell)
/// and the `custom_layout!` macro accept a `(ObjectKind, id)` tuple
/// directly.
impl<T> From<(ObjectKind, T)> for Object<T> {
    fn from((kind, id): (ObjectKind, T)) -> Self {
        Self { kind, id }
    }
}
