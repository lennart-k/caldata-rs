mod duration;
use std::fmt::Write;

pub use duration::*;
mod timezone;
use crate::rrule::{RRule, Unvalidated};
pub use timezone::*;
mod date;
mod period;
pub use date::*;
mod datetime;
pub use datetime::*;
mod dateordatetime;
pub use dateordatetime::*;
pub use period::*;
mod guess_timezone;
pub use guess_timezone::*;
mod utc_offset;
pub use utc_offset::*;

mod vcard;
pub use vcard::*;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CalDateTimeError {
    #[error(
        "Timezone has X-LIC-LOCATION property to specify a timezone from the Olson database, however its value {0} is invalid"
    )]
    InvalidOlson(String),
    #[error("TZID {0} does not refer to a valid timezone")]
    InvalidTZID(String),
    #[error("Timestamp doesn't exist because of gap in local time")]
    LocalTimeGap,
    #[error("Datetime string {0} has an invalid format")]
    InvalidDatetimeFormat(String),
    #[error("Could not parse datetime {0}")]
    ParseError(String),
    #[error("Duration string {0} has an invalid format")]
    InvalidDurationFormat(String),
    #[error("Invalid period format: {0}")]
    InvalidPeriodFormat(String),
}

pub trait Value: Sized {
    fn utc_or_local(self) -> Self {
        self
    }

    fn value_type(&self) -> Option<&'static str>;

    fn generate<W: Write>(&self, w: &mut W) -> std::fmt::Result;

    fn generate_string(&self) -> String {
        let mut out = String::new();
        self.generate(&mut out)
            .expect("Formatting to a string is infallible");
        out
    }
}

impl Value for String {
    fn value_type(&self) -> Option<&'static str> {
        Some("TEXT")
    }

    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        w.write_str(self)
    }
}

impl Value for RRule<Unvalidated> {
    fn value_type(&self) -> Option<&'static str> {
        Some("RECUR")
    }

    fn generate<W: Write>(&self, w: &mut W) -> std::fmt::Result {
        w.write_str(&self.to_string())
    }
}

impl<V: Value> Value for Vec<V> {
    fn value_type(&self) -> Option<&'static str> {
        self.first().and_then(Value::value_type)
    }

    fn utc_or_local(self) -> Self {
        self.into_iter().map(Value::utc_or_local).collect()
    }

    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        for (i, val) in self.iter().enumerate() {
            if i > 0 {
                w.write_char(',')?;
            }
            val.generate(w)?;
        }
        Ok(())
    }
}
