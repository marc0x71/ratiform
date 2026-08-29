use crate::layout::custom::CustomLayout;

pub mod builder;
pub mod custom;

/// How a form arranges each field's label relative to its value. Set via
/// [`crate::Form::with_layout`]; defaults to [`FormLayout::Horizontal`].
#[derive(Debug, Default, Clone, PartialEq)]
pub enum FormLayout<T> {
    /// Label and value side by side, on the same row — the layout used
    /// throughout this crate's examples and screenshots.
    #[default]
    Horizontal,
    /// Label above, value below, each on its own row.
    Stacked,
    /// Explicit grid of rows and columns, with full control over what
    /// each cell draws. See `CustomLayout`
    Custom(CustomLayout<T>),
}
