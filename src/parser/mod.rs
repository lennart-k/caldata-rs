mod error;
pub use error::ParserError;

mod line;
pub use line::{BytesLines, Line, LineError, LineReader};

mod content_line;
pub use content_line::{ContentLine, ContentLineError, ContentLineParams, ContentLineParser};

mod property;
pub(crate) use property::property;
pub use property::{ICalProperty, ParseProp};

mod component;
pub use component::ComponentParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserOptions {
    /// RFC 7809 allows the omission of VTIMEZONE components for standard timezones
    /// When true, we try to automatically insert missing VTIMEZONE components from the IANA
    /// timezone database.
    pub rfc7809: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self { rfc7809: false }
    }
}
