mod calendar;
pub use calendar::*;
mod calendar_object;
pub use calendar_object::*;
mod alarm;
pub use alarm::*;
mod event;
pub use event::*;
mod journal;
pub use journal::*;
mod todo;
pub use todo::*;
mod timezone;
pub use timezone::*;
mod timezone_transition;
pub use timezone_transition::*;
mod freebusy;
pub use freebusy::*;
mod timezone_object;
pub use timezone_object::*;

use crate::parser::ComponentParser;

/// Reader returning `IcalCalendar` object from a `BufRead`.
pub type IcalParser<'a, I> = ComponentParser<'a, IcalCalendar, I>;
pub type IcalObjectParser<'a, I> = ComponentParser<'a, IcalCalendarObject, I>;
pub type IcalTimeZoneObjectParser<'a, I> = ComponentParser<'a, IcalTimeZoneObject, I>;
