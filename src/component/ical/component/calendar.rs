use crate::{
    ContentLineParser,
    component::{
        self, Component, ComponentMut, IcalAlarm, IcalAlarmBuilder, IcalCalendarObject, IcalEvent,
        IcalEventBuilder, IcalFreeBusy, IcalFreeBusyBuilder, IcalJournal, IcalJournalBuilder,
        IcalTimeZone, IcalTimeZoneBuilder, IcalTodo, IcalTodoBuilder, ParserError,
    },
    generator::Emitter,
    parser::{ContentLine, ParserOptions},
    property::{
        Calscale, IcalCALSCALEProperty, IcalPRODIDProperty, IcalVERSIONProperty, IcalVersion,
    },
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
};

component::component!(
    IcalCalendar,
    "VCALENDAR",
    required_once: {
        prodid: IcalPRODIDProperty,
    },
    required_many: {
    },
    optional_many: {
    },
    optional_once: {
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
        IcalCalendarFields {
            prodid,
            other_properties,
            event,
            alarm,
            todo,
            journal,
            free_busy,
            timezone,
        }: IcalCalendarFields,
        _options: &ParserOptions,
    ) -> Result<IcalCalendar, ParserError> {
        let chrono_timezones = HashMap::from_iter(
            timezone
                .iter()
                .map(|tz| (tz.get_tzid().to_owned(), tz.into())),
        );

        Ok(IcalCalendar {
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
