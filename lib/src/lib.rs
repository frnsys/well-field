use std::{error::Error, fmt::Display};

pub use well_field_macros::*;

#[derive(Debug)]
pub struct SetFieldError {
    pub field: &'static str,
    pub received: String,
    pub expected: &'static str,
}
impl Error for SetFieldError {}
impl Display for SetFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Wrong type for field '{}': expected: {}, got: {}",
            self.field, self.expected, self.received
        )
    }
}

pub trait Fielded {
    type Field;
    type FieldValue;

    /// Set the value of the specified value.
    ///
    /// An error is returned if the value is incompatible
    /// for the specified field.
    fn set_field<V: Into<Self::FieldValue>>(
        &mut self,
        field: Self::Field,
        value: V,
    ) -> Result<(), SetFieldError>;
}
