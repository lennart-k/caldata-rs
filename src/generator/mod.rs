mod ical;
use std::collections::BTreeMap;

pub use crate::component::ical::component::{IcalCalendar, IcalEvent};
pub use crate::component::vcard::component::VcardContact;
pub use crate::parser::ContentLine;

///
/// Emits the content of the Component in ical-format.
///
pub trait Emitter {
    /// creates a textual-representation of this object and all it's properties
    /// in ical-format.
    fn generate(&self) -> String;
}

impl<K, T: Emitter> Emitter for BTreeMap<K, T> {
    fn generate(&self) -> String {
        self.values().map(Emitter::generate).collect()
    }
}

impl<T: Emitter> Emitter for Vec<T> {
    fn generate(&self) -> String {
        self.iter().map(Emitter::generate).collect()
    }
}
