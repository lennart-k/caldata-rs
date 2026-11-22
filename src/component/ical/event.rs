use crate::{
    IcalAlarm, ParserError,
    component::{Component, component},
    parser::{ContentLine, ICalProperty, ParserOptions},
    property::{
        IcalDTENDProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty, IcalDURATIONProperty,
        IcalEXDATEProperty, IcalEXRULEProperty, IcalRDATEProperty, IcalRECURIDProperty,
        IcalRRULEProperty, IcalSUMMARYProperty, IcalUIDProperty,
    },
    rrule::RRuleSet,
    types::{CalDate, CalDateOrDateTime, CalDateTime, Tz},
};
use chrono::{DateTime, Duration, Utc};
use std::collections::BTreeMap;

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

    pub fn get_effective_duration(&self) -> Option<Duration> {
        if let Some(duration) = &self.duration {
            return Some(duration.0);
        }
        if let Some(dtend) = &self.dtend {
            return Some(dtend.0.clone() - &self.dtstart.0);
        }
        None
    }

    pub fn get_last_occurence(&self) -> Option<DateTime<Tz>> {
        if self.rruleset.is_some() {
            // Non-trivial to handle
            return None;
        }
        if let Some(dtend) = &self.dtend {
            return Some(dtend.0.clone().as_datetime_ceil());
        }

        if let Some(duration) = &self.duration {
            return Some(self.dtstart.0.clone().as_datetime_ceil() + duration.0);
        }
        None
    }

    pub fn get_recurrence(&self, dtstart: CalDateOrDateTime) -> Self {
        Self {
            uid: self.uid.clone(),
            dtstamp: self.dtstamp.clone(),
            // TODO: make sure that timezone information is there
            dtstart: IcalDTSTARTProperty(dtstart.clone(), Default::default()),
            rdate: vec![],
            rrule: vec![],
            exdate: vec![],
            exrule: vec![],
            duration: self
                .get_effective_duration()
                .map(|duration| IcalDURATIONProperty(duration, Default::default())),
            dtend: None,
            recurid: Some(IcalRECURIDProperty(
                dtstart.clone(),
                Default::default(),
                crate::property::RecurIdRange::This,
            )),
            summary: self.summary.clone(),
            other_properties: self.other_properties.clone(),
            alarms: self.alarms.clone(),
            rruleset: None,
        }
    }

    /// Returns recurrence starts as datetimes.
    /// Figuring out whether the datetime actually refers to a date MUST be done afterwards.
    pub fn get_recurrence_datetimes(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: u16,
    ) -> Vec<DateTime<Tz>> {
        // Procedure:
        // - build RRuleSet (checks that UNTIL matches DTSTART)
        let Some(mut rruleset) = self.rruleset.clone() else {
            return vec![self.dtstart.0.clone().as_datetime_floor()];
        };
        // When doing datetime comparisons with local time we currently treat it as UTC
        // Since this is nontrivial to treat in rrule we add a day of tolerance to both limits and
        // TODO: do a second filtering pass where we correctly handle local time zones.
        if let Some(start) = start {
            rruleset = rruleset.after(start.with_timezone(&Tz::UTC) - Duration::days(1));
        }
        if let Some(end) = end {
            rruleset = rruleset.before(end.with_timezone(&Tz::UTC) + Duration::days(1));
        }
        rruleset.all(limit).dates
    }

    pub fn to_utc_or_local(self) -> Self {
        Self {
            dtstart: self.dtstart.utc_or_local(),
            dtend: self.dtend.map(ICalProperty::utc_or_local),
            ..self
        }
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: u16,
        overrides: Vec<Self>,
    ) -> Vec<Self> {
        let mut overrides: BTreeMap<CalDateOrDateTime, Self> = overrides
            .into_iter()
            .map(|over| (over.recurid.clone(), over))
            .filter_map(|(recurid, over)| recurid.map(|recurid| (recurid.0.clone(), over)))
            .collect();
        let is_date = self.dtstart.0.is_date();
        let recurrence_set = self
            .get_recurrence_datetimes(start, end, limit)
            .into_iter()
            .map(|datetime| {
                if is_date {
                    CalDateOrDateTime::Date(CalDate(datetime.date_naive()))
                } else {
                    CalDateOrDateTime::DateTime(CalDateTime(datetime))
                }
            });

        let mut out = vec![];
        // TODO: Support THISANDFUTURE
        for recurid in recurrence_set {
            let recurrence = if let Some(over) = overrides.remove(&recurid) {
                over
            } else {
                self.get_recurrence(recurid)
            };
            out.push(recurrence.to_utc_or_local());
        }
        // Add remaining overrides
        out.extend(overrides.into_values());
        out
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
            let rrule_dtstart: DateTime<Tz> = dtstart.0.clone().as_datetime_floor();
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
                                dates
                                    .iter()
                                    .map(|date| date.clone().start().as_datetime_floor())
                            })
                            .collect(),
                    )
                    .set_exdates(
                        exdate
                            .iter()
                            .flat_map(|IcalEXDATEProperty(dates, _)| {
                                dates.iter().map(|date| date.clone().as_datetime_floor())
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
    use chrono::Duration;

    use crate::{IcalParser, parser::ParserOptions};

    const EXAMPLE_RECUR_WHOLEDAY: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test Test Test//EN
CALSCALE:GREGORIAN
BEGIN:VEVENT
UID:ef6f3aae-0842-4c09-a8ce-4f7073d2551e
DTSTAMP:20261119T091014Z
DTSTART;VALUE=DATE:20250101
DTEND;VALUE=DATE:20250102
SUMMARY:all-day recurrence
RRULE:FREQ=DAILY;COUNT=10
END:VEVENT
END:VCALENDAR
"#;

    const EXAMPLE_RECUR_NZ: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test Test Test//EN
CALSCALE:GREGORIAN
BEGIN:VEVENT
UID:ef00abb1-11ae-4836-a782-f43a68f25fb4
DTSTAMP:20261119T091014Z
DTSTART;TZID=Pacific/Auckland:20250101T130000
DTEND;TZID=Pacific/Auckland:20250101T143000
SUMMARY:nz 13:00 - 14:30
RRULE:FREQ=DAILY;COUNT=10
END:VEVENT
END:VCALENDAR
"#;

    #[test]
    fn test_wholeday() {
        let mut cal = IcalParser::from_slice(EXAMPLE_RECUR_WHOLEDAY.as_bytes())
            .expect_one()
            .unwrap();
        let event = cal.event.pop().unwrap();
        assert!(event.dtstart.0.is_date());
        assert_eq!(event.get_effective_duration(), Some(Duration::days(1)));
        insta::assert_debug_snapshot!(event.get_recurrence_datetimes(None, None, 4), @"
        [
            2025-01-01T00:00:00Local,
            2025-01-02T00:00:00Local,
            2025-01-03T00:00:00Local,
            2025-01-04T00:00:00Local,
        ]
        ");
    }

    #[test]
    fn test_recur_nz() {
        let mut cal = IcalParser::from_slice(EXAMPLE_RECUR_NZ.as_bytes())
            .with_options(ParserOptions { rfc7809: true })
            .expect_one()
            .unwrap();
        let event = cal.event.pop().unwrap();
        assert!(!event.dtstart.0.is_date());
        assert_eq!(event.get_effective_duration(), Some(Duration::minutes(90)));
        insta::assert_debug_snapshot!(event.get_recurrence_datetimes(None, None, 4), @"
        [
            2025-01-01T13:00:00Olson(
                NZDT,
            ),
            2025-01-02T13:00:00Olson(
                NZDT,
            ),
            2025-01-03T13:00:00Olson(
                NZDT,
            ),
            2025-01-04T13:00:00Olson(
                NZDT,
            ),
        ]
        ");
    }
}
