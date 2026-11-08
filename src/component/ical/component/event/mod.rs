use crate::{
    ParserError,
    component::{Component, IcalAlarm, component},
    parser::{ContentLine, ParserOptions},
    property::{
        IcalDTENDProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty, IcalDURATIONProperty,
        IcalEXDATEProperty, IcalEXRULEProperty, IcalRDATEProperty, IcalRECURIDProperty,
        IcalRRULEProperty, IcalSUMMARYProperty, IcalUIDProperty,
    },
};
use std::collections::HashSet;

component!(
    IcalEvent,
    "VEVENT",
    required_once: {
        uid: IcalUIDProperty,
        dtstamp: IcalDTSTAMPProperty,
        dtstart: IcalDTSTARTProperty,
    },
    required_many: {
    },
    optional_many: {
        rdate: IcalRDATEProperty,
        rrule: IcalRRULEProperty,
        exdate: IcalEXDATEProperty,
        exrule: IcalEXRULEProperty,
    },
    optional_once: {
        duration: IcalDURATIONProperty,
        dtend: IcalDTENDProperty,
        recurid: IcalRECURIDProperty,
        summary: IcalSUMMARYProperty,
    },
    components: {
        alarms: IcalAlarm
    },
    extra_fields: {
        rruleset: Option<crate::rrule::RRuleSet>
    },
);

impl IcalEvent {
    pub fn get_tzids(&self) -> HashSet<&str> {
        todo!()
    }
}

impl IcalEventBuilder {
    fn build_from_fields(
        IcalEventFields {
            uid,
            dtstamp,
            dtstart,
            rdate,
            rrule,
            exdate,
            exrule,
            duration,
            dtend,
            recurid,
            summary,
            other_properties,
            alarms,
        }: IcalEventFields,
        _: &ParserOptions,
    ) -> Result<IcalEvent, ParserError> {
        Ok(IcalEvent {
            uid,
            dtstamp,
            dtstart,
            rdate,
            rrule,
            exdate,
            exrule,
            duration,
            dtend,
            recurid,
            summary,
            other_properties,
            alarms,
            rruleset: None,
        })
    }
}
