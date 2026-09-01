//! The error type returned by [`FormBuilder::build`](crate::builder::FormBuilder::build)
//! when the fields as constructed can't be turned into a valid `FormState`.

use std::fmt;

/// Why [`FormBuilder::build`](crate::builder::FormBuilder::build) refused to
/// build the form.
///
/// Refers to fields and values by their position (the order they were
/// added in), not by id — this keeps `BuildError` non-generic, so it needs
/// no bound on your id type beyond what the rest of the crate already
/// requires (`PartialEq`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildError {
    /// Two fields were given the same id. `first` and `duplicate` are the
    /// positions of both fields in the order they were added
    DuplicateFieldId {
        /// Position of the first field with this id.
        first: usize,
        /// Position of the field that repeats it.
        duplicate: usize,
    },
    /// A `Select` or `MultiSelect` field has two options with the same value. `first` and
    /// `duplicate` are positions within that field's own list of values
    DuplicateSelectValue {
        /// Position of the first option with this value.
        first: usize,
        /// Position of the option that repeats it.
        duplicate: usize,
    },
    /// A `MultiSelect` field has an option whose value contains the `,`
    /// separator used to encode multiple selections into a single `String`.
    /// `position` is where that option sits in the field's own list of values.
    InvalidMultiSelectValue {
        /// Position of the offending option within its field's list of values.
        position: usize,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::DuplicateFieldId { first, duplicate } => {
                write!(
                    f,
                    "field at index {duplicate} has the same id as the field at index {first}"
                )
            }
            BuildError::DuplicateSelectValue { first, duplicate } => {
                write!(
                    f,
                    "Select option at index {duplicate} has the same value as the option at index {first}"
                )
            }
            BuildError::InvalidMultiSelectValue { position } => {
                write!(
                    f,
                    "MultiSelect option contains an invalid value at {position}"
                )
            }
        }
    }
}

impl std::error::Error for BuildError {}
