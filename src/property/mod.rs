use crate::{
    parser::{ContentLine, property},
    types::PartialDateAndOrTime,
    types::UtcOffset,
};

mod duration;
pub use duration::*;
mod exdate;
pub use exdate::*;
mod rdate;
pub use rdate::*;
mod dtstart;
pub use dtstart::*;
mod recurid;
pub use recurid::*;
mod due;
pub use due::*;
mod dtstamp;
pub use dtstamp::*;
mod dtend;
pub use dtend::*;
mod calscale;
pub use calscale::*;
mod version;
pub use version::*;

property!("UID", "TEXT", IcalUIDProperty, String);

impl From<String> for IcalUIDProperty {
    fn from(value: String) -> Self {
        Self(value, Default::default())
    }
}

property!("SUMMARY", "TEXT", IcalSUMMARYProperty, String);

property!(
    "RRULE",
    "RECUR",
    IcalRRULEProperty,
    crate::rrule::RRule<crate::rrule::Unvalidated>
);
property!(
    "EXRULE",
    "RECUR",
    IcalEXRULEProperty,
    crate::rrule::RRule<crate::rrule::Unvalidated>
);
property!("PRODID", "TEXT", IcalPRODIDProperty, String);

property!("METHOD", "TEXT", IcalMETHODProperty, String);

property!("TZID", "TEXT", IcalTZIDProperty, String);
property!("X-LIC-LOCATION", "TEXT", IcalXLICLOCATIONProperty, String);
property!("TZNAME", "TEXT", IcalTZNAMEProperty, String);

property!(
    "TZOFFSETFROM",
    "UTC-OFFSET",
    IcalTZOFFSETFROMProperty,
    UtcOffset
);
property!(
    "TZOFFSETTO",
    "UTC-OFFSET",
    IcalTZOFFSETTOProperty,
    UtcOffset
);

property!("UID", "TEXT", VcardUIDProperty, String);
property!("FN", "TEXT", VcardFNProperty, String);
property!("N", "TEXT", VcardNProperty, String);
property!("NICKNAME", "TEXT", VcardNICKNAMEProperty, String);
property!(
    "BDAY",
    "DATE-AND-OR-TIME",
    VcardBDAYProperty,
    PartialDateAndOrTime
);
property!(
    "ANNIVERSARY",
    "DATE-AND-OR-TIME",
    VcardANNIVERSARYProperty,
    PartialDateAndOrTime
);
