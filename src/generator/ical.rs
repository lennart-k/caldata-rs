use crate::component::IcalCalendarObject;
use crate::component::{
    IcalAlarm, IcalCalendar, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone,
    IcalTimeZoneTransition, IcalTodo,
};
use crate::generator::Emitter;

impl Emitter for IcalTimeZoneTransition {
    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        let compname = crate::component::Component::get_comp_name(self);
        w.write_str("BEGIN:")?;
        w.write_str(compname)?;
        w.write_str("\r\n")?;
        for property in &self.properties {
            property.generate(w)?;
        }
        w.write_str("END:")?;
        w.write_str(compname)?;
        w.write_str("\r\n")?;
        Ok(())
    }
}

macro_rules! generate_emitter {
    ($struct:ty, $($child:ident),*) => {
        impl Emitter for $struct {
            fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
                let compname = crate::component::Component::get_comp_name(self);
                w.write_str("BEGIN:")?;
                w.write_str(compname)?;
                w.write_str("\r\n")?;
                // for property in &self.properties {
                //     property.generate(w)?;
                // }
                // $(self.$child.generate(w)?;)*
                w.write_str("END:")?;
                w.write_str(compname)?;
                w.write_str("\r\n")?;
                Ok(())
            }
        }
    };
}

use crate::component::VcardContact;
generate_emitter!(VcardContact,);

generate_emitter!(IcalAlarm,);
generate_emitter!(IcalFreeBusy,);
generate_emitter!(IcalJournal,);
generate_emitter!(IcalEvent, alarms);
generate_emitter!(IcalTimeZone<true>, transitions);
generate_emitter!(
    IcalCalendar,
    vtimezones,
    events,
    alarms,
    todos,
    journals,
    free_busys
);
generate_emitter!(IcalCalendarObject, vtimezones, inner);
