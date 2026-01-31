use crate::{
    component::Component,
    parser::{ContentLine, ICalProperty, ParserError, property},
    types::PartialDateAndOrTime,
};
use std::collections::HashMap;

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

pub trait GetProperty: Component {
    fn safe_get_all<T: ICalProperty>(
        &self,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Vec<T>, ParserError> {
        self.get_named_properties(T::NAME)
            .map(|prop| ICalProperty::parse_prop(prop, timezones))
            .collect::<Result<Vec<_>, _>>()
    }

    fn safe_get_optional<T: ICalProperty>(
        &self,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Option<T>, ParserError> {
        let mut props = self.get_named_properties(T::NAME);
        let Some(prop) = props.next() else {
            return Ok(None);
        };
        if props.next().is_some() {
            return Err(ParserError::PropertyConflict(
                "Multiple instances of property",
            ));
        }
        ICalProperty::parse_prop(prop, timezones).map(Some)
    }

    fn safe_get_required<T: ICalProperty>(
        &self,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<T, ParserError> {
        self.safe_get_optional(timezones)?
            .ok_or(ParserError::MissingProperty(T::NAME))
    }

    fn has_prop<T: ICalProperty>(&self) -> bool {
        self.get_property(T::NAME).is_some()
    }
}

impl<C: Component> GetProperty for C {}

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
