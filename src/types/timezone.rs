use chrono::{MappedLocalTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use derive_more::{Display, From};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq)]
pub enum Tz {
    Local,
    Olson(chrono_tz::Tz),
}

impl Tz {
    pub const UTC: Self = Self::Olson(chrono_tz::UTC);

    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local)
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Local => "Local",
            Self::Olson(tz) => tz.name(),
        }
    }

    pub fn utc() -> Self {
        Self::Olson(chrono_tz::UTC)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum CalTimezoneOffset {
    Local,
    Olson(chrono_tz::TzOffset),
}

impl chrono::Offset for CalTimezoneOffset {
    fn fix(&self) -> chrono::FixedOffset {
        match self {
            Self::Local => Utc.fix(),
            Self::Olson(olson) => olson.fix(),
        }
    }
}

impl TimeZone for Tz {
    type Offset = CalTimezoneOffset;

    fn from_offset(offset: &Self::Offset) -> Self {
        match offset {
            CalTimezoneOffset::Local => Self::Local,
            CalTimezoneOffset::Olson(offset) => Self::Olson(chrono_tz::Tz::from_offset(offset)),
        }
    }

    #[cfg(not(tarpaulin_include))] // Only used by deprecated chrono::Date type
    fn offset_from_local_date(&self, local: &NaiveDate) -> chrono::MappedLocalTime<Self::Offset> {
        match self {
            Self::Local => MappedLocalTime::Single(CalTimezoneOffset::Local),
            Self::Olson(tz) => tz
                .offset_from_local_date(local)
                .map(CalTimezoneOffset::Olson),
        }
    }

    fn offset_from_local_datetime(
        &self,
        local: &NaiveDateTime,
    ) -> chrono::MappedLocalTime<Self::Offset> {
        match self {
            Self::Local => MappedLocalTime::Single(CalTimezoneOffset::Local),
            Self::Olson(tz) => tz
                .offset_from_local_datetime(local)
                .map(CalTimezoneOffset::Olson),
        }
    }

    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> Self::Offset {
        match self {
            Self::Local => CalTimezoneOffset::Local,
            Self::Olson(tz) => CalTimezoneOffset::Olson(tz.offset_from_utc_datetime(utc)),
        }
    }

    #[cfg(not(tarpaulin_include))] // Only used by deprecated chrono::Date type
    fn offset_from_utc_date(&self, utc: &NaiveDate) -> Self::Offset {
        match self {
            Self::Local => CalTimezoneOffset::Local,
            Self::Olson(tz) => CalTimezoneOffset::Olson(tz.offset_from_utc_date(utc)),
        }
    }
}
