pub mod samples;
pub mod tracer;
pub mod types;

pub use tracer::Tracer;

/// Configure how the schema is traced
///
/// Example:
///
/// ```rust
/// # use serde_arrow::schema::TracingOptions;
/// let tracing_options = TracingOptions::default()
///     .map_as_struct(true)
///     .string_dictionary_encoding(false);
/// ```
///
/// The defaults are:
///
/// ```rust
/// # use serde_arrow::schema::TracingOptions;
/// let default = TracingOptions::default();
///
/// assert_eq!(default.allow_null_fields, false);
/// assert_eq!(default.map_as_struct, true);
/// assert_eq!(default.string_dictionary_encoding, false);
/// assert_eq!(default.coerce_numbers, false);
/// assert_eq!(default.guess_dates, false);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TracingOptions {
    /// If `true`, accept null-only fields (e.g., fields with type `()` or fields
    /// with only `None` entries). If `false`, schema tracing will fail in this
    /// case.
    pub allow_null_fields: bool,

    /// If `true` serialize maps as structs (the default). See
    /// [`Strategy::MapAsStruct`][crate::schema::Strategy] for details.
    pub map_as_struct: bool,

    /// If `true` serialize strings dictionary encoded. The default is `false`.
    ///
    /// If `true`, strings are traced as `Dictionary(UInt64, LargeUtf8)`. If
    /// `false`, strings are traced as `LargeUtf8`.
    pub string_dictionary_encoding: bool,

    /// If `true`, coerce different numeric types.
    ///
    /// This option may be helpful when dealing with data formats that do not
    /// encode the complete numeric type, e.g., JSON. The following rules are
    /// used:
    ///
    /// - unsigned + other unsigned -> u64
    /// - signed + other signed -> i64
    /// - float + other float -> f64
    /// - unsigned + signed -> i64
    /// - unsigned + float -> f64
    /// - signed  + float -> f64
    pub coerce_numbers: bool,

    /// If `true`, try to auto detect datetimes in string columns
    ///
    /// Currently the naive datetime (`YYYY-MM-DDThh:mm:ss`) and UTC datetimes
    /// (`YYYY-MM-DDThh:mm:ssZ`) are understood.
    ///
    /// For string fields where all values are either missing or conform to one
    /// of the format the data type is set as `Date64` with strategy
    /// [`NaiveStrAsDate64`][crate::schema::Strategy::NaiveStrAsDate64] or
    /// [`UtcStrAsDate64`][crate::schema::Strategy::UtcStrAsDate64].
    pub guess_dates: bool,

    /// If not `None`, trace the schema as a field with the given name instead
    /// of multiple fields
    ///
    /// This may be helpful when the individual items are not structs, but other
    /// objects, e.g., numbers or strings.
    pub as_field: Option<String>,
}

impl Default for TracingOptions {
    fn default() -> Self {
        Self {
            allow_null_fields: false,
            map_as_struct: true,
            string_dictionary_encoding: false,
            coerce_numbers: false,
            guess_dates: false,
            as_field: None,
        }
    }
}

impl TracingOptions {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set [`allow_null_fields`](#structfield.allow_null_fields)
    pub fn allow_null_fields(mut self, value: bool) -> Self {
        self.allow_null_fields = value;
        self
    }

    /// Set [`map_as_struct`](#structfield.map_as_struct)
    pub fn map_as_struct(mut self, value: bool) -> Self {
        self.map_as_struct = value;
        self
    }

    /// Set [`string_dictionary_encoding`](#structfield.string_dictionary_encoding)
    pub fn string_dictionary_encoding(mut self, value: bool) -> Self {
        self.string_dictionary_encoding = value;
        self
    }

    /// Set [`coerce_numbers`](#structfield.coerce_numbers)
    pub fn coerce_numbers(mut self, value: bool) -> Self {
        self.coerce_numbers = value;
        self
    }

    /// Set [`try_parse_dates`](#structfield.try_parse_dates)
    pub fn guess_dates(mut self, value: bool) -> Self {
        self.guess_dates = value;
        self
    }

    /// Set [`as_field`](#structfield.as_field)
    pub fn as_field<S: Into<String>>(mut self, value: S) -> Self {
        self.as_field = Some(value.into());
        self
    }
}
