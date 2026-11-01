use crate::{
    ParserError,
    parser::{ContentLine, property},
    types::PartialDateAndOrTime,
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

pub trait ParseContentLine: Sized {
    fn from_content_line(
        content_line: ContentLine,
        timezones: Option<&std::collections::HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self, ParserError>;
}

macro_rules! property_enum {
    ($name:ident, $(($prop_variant:ident, $prop_type:ty)),*) => {
        ::paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum [<$name Name>] {
                $($prop_variant),*,
                Other(String),
            }

            impl From<&str> for [<$name Name>] {
                fn from(name: &str) -> Self {
                    match name {
                        $(<$prop_type as crate::parser::ICalProperty>::NAME => Self::$prop_variant),*,
                        other => Self::Other(other.to_string())
                    }
                }
            }

            #[derive(Debug, Clone, derive_more::From)]
            pub enum $name {
                $($prop_variant($prop_type)),*,
                Other(crate::parser::ContentLine)
            }

            impl $name {
                pub fn name(&self) -> [<$name Name>] {
                    match self {
                        $(Self::$prop_variant(_) => [<$name Name>]::$prop_variant),*,
                        Self::Other(content_line) => [<$name Name>]::Other(content_line.name.clone())
                    }
                }
            }

            impl crate::property::ParseContentLine for $name {
                fn from_content_line(
                    content_line: ContentLine,
                    timezones: Option<&std::collections::HashMap<String, Option<chrono_tz::Tz>>>,
                ) -> Result<Self, ParserError> {
                    let property_name: [<$name Name>] = content_line.name.as_str().into();

                    Ok(match property_name {
                        $([<$name Name>]::$prop_variant => $name::$prop_variant(crate::parser::ICalProperty::parse_prop(&content_line, timezones)?)),*,
                        [<$name Name>]::Other(_) => $name::Other(content_line)
                    })
                }
            }

            impl From<$name> for ContentLine {
                fn from(val: $name) -> ContentLine {
                    match val {
                        $($name::$prop_variant(prop) => prop.into()),*,
                        $name::Other(content_line) => content_line
                    }
                }
            }
        }
    };
}
pub(super) use property_enum;
