//! Parse an ICAL calendar.
//!
//! Wrap the result of the `PropertyParser` into components.
//!
//! Each component contains properties (ie: `Property`) or sub-components.
//!
//! * The `VcardParser` return `IcalCalendar` objects.
//!
//! # Examples
//!
//! ```rust
//! use std::fs::read_to_string;
//!
//! let buf = read_to_string("./tests/resources/ical_multiple.ics")
//! .unwrap();
//!
//! let reader = caldata::IcalParser::from_slice(buf.as_bytes());
//!
//! for line in reader {
//!     println!("{:?}", line);
//! }
//! ```

pub mod component;
use component::IcalCalendar;

use super::IcalCalendarObject;
use crate::parser::ComponentParser;

/// Reader returning `IcalCalendar` object from a `BufRead`.
pub type IcalParser<'a, I> = ComponentParser<'a, IcalCalendar, I>;
pub type IcalObjectParser<'a, I> = ComponentParser<'a, IcalCalendarObject, I>;
