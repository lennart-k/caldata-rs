use crate::{
    component::{
        self, Component, IcalAlarm, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone, IcalTodo,
        ParserError,
    },
    parser::{ContentLine, ParserOptions},
    property::{IcalCALSCALEProperty, IcalPRODIDProperty, IcalVERSIONProperty},
};
use std::collections::{HashMap, HashSet};

component::component!(
    IcalCalendar,
    "VCALENDAR",
    required_once: {
        version: IcalVERSIONProperty,
        prodid: IcalPRODIDProperty,
    },
    required_many: {
    },
    optional_many: {
    },
    optional_once: {
        calscale: IcalCALSCALEProperty,
    },
    components: {
        event: IcalEvent,
        alarm: IcalAlarm,
        todo: IcalTodo,
        journal: IcalJournal,
        free_busy: IcalFreeBusy,
        timezone: IcalTimeZone,
    },
    extra_fields: {
        chrono_timezones: HashMap<String, Option<chrono_tz::Tz>>
    },
);

impl IcalCalendarBuilder {
    fn build_from_fields(
        fields: IcalCalendarFields,
        _options: &ParserOptions,
    ) -> Result<IcalCalendar, ParserError> {
        let _tzids: HashSet<&str> = fields.get_tzids().collect();

        let IcalCalendarFields {
            version,
            calscale,
            prodid,
            other_properties,
            event,
            alarm,
            todo,
            journal,
            free_busy,
            timezone,
        } = fields;

        let chrono_timezones = HashMap::from_iter(
            timezone
                .iter()
                .map(|tz| (tz.get_tzid().to_owned(), tz.into())),
        );

        Ok(IcalCalendar {
            version,
            calscale,
            prodid,
            other_properties,
            event,
            alarm,
            todo,
            journal,
            free_busy,
            chrono_timezones,
            timezone,
        })
    }
}

impl IcalCalendar {
    pub fn get_timezones(&self) -> &[IcalTimeZone] {
        &self.timezone
    }
}
