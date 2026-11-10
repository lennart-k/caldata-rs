use chrono::{DateTime, Utc};

use crate::{
    ParserError,
    component::{Component, IcalAlarm, component},
    parser::{ContentLine, ICalProperty, ParserOptions},
    property::{
        IcalDTENDProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty, IcalDURATIONProperty,
        IcalEXDATEProperty, IcalEXRULEProperty, IcalRDATEProperty, IcalRECURIDProperty,
        IcalRRULEProperty, IcalSUMMARYProperty, IcalUIDProperty,
    },
    rrule::RRuleSet,
    types::Tz,
};

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
    pub fn get_uid(&self) -> &str {
        &self.uid.0
    }

    pub fn get_recurid(&self) -> Option<&IcalRECURIDProperty> {
        self.recurid.as_ref()
    }

    pub fn expand_recurrence(
        mut self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        overrides: &[Self],
    ) -> Vec<Self> {
        let mut overrides = overrides.to_vec();

        vec![Self {
            rdate: vec![],
            rrule: vec![],
            exdate: vec![],
            exrule: vec![],
            ..self
        }]
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
        let rruleset = if !(rrule.is_empty() && rdate.is_empty()) {
            let rrule_dtstart: DateTime<Tz> = dtstart.0.clone().into();
            Some(
                RRuleSet::new(rrule_dtstart)
                    .set_rrules(
                        rrule
                            .iter()
                            .map(|rrule| rrule.0.clone().validate(rrule_dtstart))
                            .collect::<Result<_, _>>()?,
                    )
                    .set_exrules(
                        exrule
                            .iter()
                            .map(|exrule| exrule.0.clone().validate(rrule_dtstart))
                            .collect::<Result<_, _>>()?,
                    )
                    .set_rdates(
                        rdate
                            .iter()
                            .flat_map(|IcalRDATEProperty(dates, _)| {
                                // TODO: Support periods
                                dates.iter().map(|date| date.clone().start().into())
                            })
                            .collect(),
                    )
                    .set_exdates(
                        exdate
                            .iter()
                            .flat_map(|IcalEXDATEProperty(dates, _)| {
                                dates.iter().map(|date| date.clone().into())
                            })
                            .collect(),
                    ),
            )
        } else {
            None
        };

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
            rruleset,
        })
    }
}

#[cfg(test)]
mod tests {
    const EXAMPLE_EVENT: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
CALSCALE:GREGORIAN
PRODID:-//SabreDAV//SabreDAV//EN
BEGIN:VEVENT
DTSTAMP:20201204T082107Z
UID:6bec1263-c5f4-4446-8a0c-10d960a6e06e
DTSTART;TZID=CET:20201225T150000
DTEND;TZID=Europe/Berlin:20201225T223000
SUMMARY:Omit VTIMEZONE
END:VEVENT
END:VCALENDAR
"#;
    #[test]
    fn test_recurrence_expansion() {}
}
