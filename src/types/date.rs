use crate::parser::ContentLine;
use crate::types::{CalDateTimeError, Value};
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

pub const LOCAL_DATE: &str = "%Y%m%d";

#[derive(Debug, Clone, PartialEq, Eq)]
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

    #[must_use]
    pub fn naive_date(&self) -> &NaiveDate {
        &self.0
    }

    #[must_use]
    pub fn format(&self) -> String {
        self.0.format(LOCAL_DATE).to_string()
    }

    pub fn parse(value: &str) -> Result<Self, CalDateTimeError> {
        if let Ok(date) = NaiveDate::parse_from_str(value, LOCAL_DATE) {
            return Ok(Self(date));
        }
        Err(CalDateTimeError::InvalidDatetimeFormat(value.to_string()))
    }

    #[must_use]
    pub fn succ_opt(&self) -> Option<Self> {
        Some(Self(self.0.succ_opt()?))
    }
}

#[cfg(not(tarpaulin_include))]
impl Datelike for CalDate {
    fn year(&self) -> i32 {
        self.0.year()
    }
    fn month(&self) -> u32 {
        self.0.month()
    }

    fn month0(&self) -> u32 {
        self.0.month0()
    }
    fn day(&self) -> u32 {
        self.0.day()
    }
    fn day0(&self) -> u32 {
        self.0.day0()
    }
    fn ordinal(&self) -> u32 {
        self.0.ordinal()
    }
    fn ordinal0(&self) -> u32 {
        self.0.ordinal0()
    }
    fn weekday(&self) -> chrono::Weekday {
        self.0.weekday()
    }
    fn iso_week(&self) -> chrono::IsoWeek {
        self.0.iso_week()
    }
    fn with_year(&self, year: i32) -> Option<Self> {
        Some(Self(self.0.with_year(year)?))
    }
    fn with_month(&self, month: u32) -> Option<Self> {
        Some(Self(self.0.with_month(month)?))
    }
    fn with_month0(&self, month0: u32) -> Option<Self> {
        Some(Self(self.0.with_month0(month0)?))
    }
    fn with_day(&self, day: u32) -> Option<Self> {
        Some(Self(self.0.with_day(day)?))
    }
    fn with_day0(&self, day0: u32) -> Option<Self> {
        Some(Self(self.0.with_day0(day0)?))
    }
    fn with_ordinal(&self, ordinal: u32) -> Option<Self> {
        Some(Self(self.0.with_ordinal(ordinal)?))
    }
    fn with_ordinal0(&self, ordinal0: u32) -> Option<Self> {
        Some(Self(self.0.with_ordinal0(ordinal0)?))
    }
}

impl Value for CalDate {
    fn value_type(&self) -> Option<&'static str> {
        Some("DATE")
    }
    fn value(&self) -> String {
        self.format()
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
