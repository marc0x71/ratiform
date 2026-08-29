use std::fmt;

/// Why [`crate::builder::FormBuilder::build`] refused to build the form. Refers to fields
/// by their position in the order they were added — the same order
/// [`crate::FormState::values`] and friends walk them in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildError {
    /// The field at `duplicate` has the same id as the field at `first`
    /// (`first < duplicate`).
    DuplicateFieldId { first: usize, duplicate: usize },
    /// The `Select` field has two options with the same
    /// value — the one at `duplicate` repeats the one already at `first`
    /// (`first < duplicate`, both indices into that field's own list of
    /// values, not into the form's fields).
    DuplicateSelectValue { first: usize, duplicate: usize },
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
                    "Select has an option at index {duplicate} \
                     with the same value as the option at index {first}"
                )
            }
        }
    }
}

impl std::error::Error for BuildError {}
