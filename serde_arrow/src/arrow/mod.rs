//! Support for the arrow crate (requires one the `arrow-*` features)
//!
mod schema;
mod sinks;

use serde::Serialize;

use crate::{
    impls::arrow::{array::ArrayRef, schema::Field},
    internal::{
        self,
        error::Result,
        schema::{GenericField, TracingOptions},
    },
};

use self::sinks::ArrowPrimitiveBuilders;

/// Determine the schema (as a list of fields) for the given items
///
/// `items` should be given in the form a list of records (e.g., a vector of
/// structs).
///
/// To correctly record the type information make sure to:
///
/// - include values for `Option<T>`
/// - include all variants of an enum
/// - include at least single element of a list or a map
///
pub fn serialize_into_fields<T>(items: &T, options: TracingOptions) -> Result<Vec<Field>>
where
    T: Serialize + ?Sized,
{
    internal::serialize_into_fields(items, options)?
        .iter()
        .map(|f| f.try_into())
        .collect()
}

/// Determine the schema of an object that represents a single array
///
pub fn serialize_into_field<T>(items: &T, name: &str, options: TracingOptions) -> Result<Field>
where
    T: Serialize + ?Sized,
{
    let field = internal::serialize_into_field(items, name, options)?;
    (&field).try_into()
}

/// Build arrays from the given items
///
/// `items` should be given in the form a list of records (e.g., a vector of
/// structs).
///
/// To build arrays record by record use [ArraysBuilder].
///
pub fn serialize_into_arrays<T>(fields: &[Field], items: &T) -> Result<Vec<ArrayRef>>
where
    T: Serialize + ?Sized,
{
    let fields = fields
        .iter()
        .map(GenericField::try_from)
        .collect::<Result<Vec<_>>>()?;
    internal::serialize_into_arrays::<T, ArrowPrimitiveBuilders>(&fields, items)
}

/// Serialize an object that represents a single array into an array
///
///
pub fn serialize_into_array<T>(field: &Field, items: &T) -> Result<ArrayRef>
where
    T: Serialize + ?Sized,
{
    let field: GenericField = field.try_into()?;
    internal::serialize_into_array::<T, ArrowPrimitiveBuilders>(&field, items)
}
