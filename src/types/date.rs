use crate::parser::ContentLine;
use crate::types::{CalDateTimeError, Value};
use chrono::NaiveDate;
use std::collections::HashMap;

pub const LOCAL_DATE: &str = "%Y%m%d";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CalDate(pub NaiveDate);

impl PartialOrd for CalDate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CalDate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl CalDate {
    pub fn parse_prop(
        prop: &ContentLine,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self, CalDateTimeError> {
        Self::parse(&prop.value)
    }

    pub fn parse(value: &str) -> Result<Self, CalDateTimeError> {
        if let Ok(date) = NaiveDate::parse_from_str(value, LOCAL_DATE) {
            return Ok(Self(date));
        }
        Err(CalDateTimeError::InvalidDatetimeFormat(value.to_string()))
    }
}

impl Value for CalDate {
    fn value_type(&self) -> Option<&'static str> {
        Some("DATE")
    }

    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        self.0.format(LOCAL_DATE).write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::CalDate;

    #[test]
    fn test_date() {
        let a = CalDate::parse("20121212").unwrap();
        let b = CalDate::parse("20121213").unwrap();
        assert!(b > a);
        assert!(b >= a);
    }
}
