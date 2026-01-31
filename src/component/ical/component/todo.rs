use crate::rrule::RRule;

use crate::types::Tz;
use crate::{
    ContentLineParser,
    component::{Component, ComponentMut, IcalAlarm, IcalAlarmBuilder},
    parser::{ContentLine, ParserError, ParserOptions},
    property::{
        GetProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty, IcalDUEProperty,
        IcalDURATIONProperty, IcalEXDATEProperty, IcalEXRULEProperty, IcalRDATEProperty,
        IcalRECURIDProperty, IcalRRULEProperty, IcalUIDProperty,
    },
    types::CalDateOrDateTime,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

#[derive(Debug, Clone)]
pub struct IcalTodo {
    uid: String,
    pub dtstart: Option<IcalDTSTARTProperty>,
    pub due: Option<IcalDUEProperty>,
    pub duration: Option<IcalDURATIONProperty>,
    pub dtstamp: IcalDTSTAMPProperty,
    pub properties: Vec<ContentLine>,
    pub alarms: Vec<IcalAlarm>,
    rdates: Vec<IcalRDATEProperty>,
    rrules: Vec<RRule>,
    exdates: Vec<IcalEXDATEProperty>,
    exrules: Vec<RRule>,
    pub(crate) recurid: Option<IcalRECURIDProperty>,
}

#[derive(Debug, Clone, Default)]
pub struct IcalTodoBuilder {
    pub properties: Vec<ContentLine>,
    pub alarms: Vec<IcalAlarmBuilder>,
}

impl IcalTodo {
    pub fn get_uid(&self) -> &str {
        &self.uid
    }

    pub fn has_rruleset(&self) -> bool {
        !self.rrules.is_empty()
            || !self.rdates.is_empty()
            || !self.exrules.is_empty()
            || !self.exdates.is_empty()
    }

    pub fn get_alarms(&self) -> &[IcalAlarm] {
        &self.alarms
    }

    pub fn get_last_occurence(&self) -> Option<CalDateOrDateTime> {
        if self.has_rruleset() {
            // Non-trivial to handle
            return None;
        }
        if let Some(dtend) = &self.due {
            return Some(dtend.0.clone());
        }

        if let Some(dtstart) = &self.dtstart
            && let Some(duration) = &self.duration
        {
            return Some((dtstart.0.clone() + duration.0).into());
        }

        None
    }
}

impl Component for IcalTodo {
    const NAMES: &[&str] = &["VTODO"];
    type Unverified = IcalTodoBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTodoBuilder {
            properties: self.properties,
            alarms: self
                .alarms
                .into_iter()
                .map(|alarm| alarm.mutable())
                .collect(),
        }
    }
}

impl Component for IcalTodoBuilder {
    const NAMES: &[&str] = &["VTODO"];
    type Unverified = IcalTodoBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl ComponentMut for IcalTodoBuilder {
    type Verified = IcalTodo;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    #[inline]
    fn add_sub_component<'a, I: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        line_parser: &mut ContentLineParser<'a, I>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarmBuilder::new();
                alarm.parse(line_parser, options)?;
                self.alarms.push(alarm);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        options: &ParserOptions,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalTodo, ParserError> {
        // REQUIRED, but ONLY ONCE
        let IcalUIDProperty(uid, _) = self.safe_get_required(timezones)?;
        let dtstamp = self.safe_get_required(timezones)?;

        // OPTIONAL, but ONLY ONCE: class / completed / created / description / dtstart / geo / last-mod / location / organizer / percent / priority / recurid / seq / status / summary / url / rrule
        let dtstart = self.safe_get_optional::<IcalDTSTARTProperty>(timezones)?;
        let recurid = self.safe_get_optional::<IcalRECURIDProperty>(timezones)?;
        if let Some(IcalDTSTARTProperty(dtstart, _)) = &dtstart
            && let Some(recurid) = &recurid
        {
            recurid.validate_dtstart(dtstart)?;
        }
        // OPTIONAL, but MUTUALLY EXCLUSIVE
        let duration = self.safe_get_optional::<IcalDURATIONProperty>(timezones)?;
        let due = self.safe_get_optional::<IcalDUEProperty>(timezones)?;
        if duration.is_some() && due.is_some() {
            return Err(ParserError::PropertyConflict(
                "both DUE and DURATION are defined",
            ));
        }

        // OPTIONAL, MULTIPLE ALLOWED: attach / attendee / categories / comment / contact / exdate / rstatus / related / resources / rdate / x-prop / iana-prop
        let rdates = self.safe_get_all::<IcalRDATEProperty>(timezones)?;
        let exdates = self.safe_get_all::<IcalEXDATEProperty>(timezones)?;
        let (rrules, exrules) = if let Some(dtstart) = dtstart.as_ref() {
            let dtstart = dtstart.0.utc().with_timezone(&Tz::UTC);
            let rrules = self
                .safe_get_all::<IcalRRULEProperty>(timezones)?
                .into_iter()
                .map(|rrule| rrule.0.validate(dtstart))
                .collect::<Result<Vec<_>, _>>()?;
            let exrules = self
                .safe_get_all::<IcalEXRULEProperty>(timezones)?
                .into_iter()
                .map(|rrule| rrule.0.validate(dtstart))
                .collect::<Result<Vec<_>, _>>()?;
            (rrules, exrules)
        } else {
            (vec![], vec![])
        };

        let verified = IcalTodo {
            uid,
            dtstamp,
            dtstart,
            due,
            duration,
            rdates,
            rrules,
            exdates,
            exrules,
            recurid,
            properties: self.properties,
            alarms: self
                .alarms
                .into_iter()
                .map(|alarm| alarm.build(options, timezones))
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(verified)
    }
}

impl IcalTodo {
    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .chain(self.alarms.iter().flat_map(IcalAlarm::get_tzids))
            .collect()
    }
}

impl IcalTodoBuilder {
    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .collect()
    }
}
