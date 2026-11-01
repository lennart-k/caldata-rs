use chrono::{DateTime, Utc};

use crate::component::vcard::component;
use crate::property::{
    IcalDTSTAMPProperty, IcalDTSTARTProperty, IcalDUEProperty, IcalDURATIONProperty,
};
use crate::rrule::{self, RRuleSet};
use crate::types::CalDateTime;
use crate::{
    component::{Component, ComponentMut, IcalAlarm},
    parser::{ContentLine, ParserError, ParserOptions},
    property::{
        IcalEXDATEProperty, IcalEXRULEProperty, IcalRDATEProperty, IcalRECURIDProperty,
        IcalRRULEProperty, IcalUIDProperty,
    },
};
use std::collections::{HashMap, HashSet};

component::component!(
    IcalTodo,
    &["VTODO"],
    required_once: {
        uid: IcalUIDProperty,
        dtstamp: IcalDTSTAMPProperty,
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
        dtstart: IcalDTSTARTProperty,
        duration: IcalDURATIONProperty,
        due: IcalDUEProperty,
        recurid: IcalRECURIDProperty,
    },
    components: {
        alarms: IcalAlarm
    },
    extra_fields: {
        rruleset: Option<crate::rrule::RRuleSet> = |&dtstart, &rrule, &rdate, &exrule, &exdate| {
            let dtstart: &Option<IcalDTSTARTProperty> = dtstart;
            let rrule: &Option<IcalRRULEProperty> = rrule;
            let rdate: &Option<IcalRDATEProperty> = rdate;
            let exrule: &Option<IcalEXRULEProperty> = exrule;
            let exdate: &Option<IcalEXDATEProperty> = exdate;
            let Some(dtstart) = dtstart.as_ref() else {
                return None;
            };
            let rrule_dtstart: DateTime<crate::types::Tz> = dtstart.0.clone().into();
            let rrules = rrule.iter().map(|rrule| rrule.0.clone().validate(rrule_dtstart))
                .collect::<Result<Vec<_>, _>>().ok()?;
            Some(crate::rrule::RRuleSet::new(rrule_dtstart))
        }
    },
);

impl IcalTodo {
    pub fn get_uid(&self) -> &str {
        &self.uid.0
    }

    pub fn has_rruleset(&self) -> bool {
        self.rruleset.is_some()
    }

    pub fn get_alarms(&self) -> &[IcalAlarm] {
        &self.alarms
    }

    pub fn get_last_occurence(&self) -> Option<CalDateTime> {
        None
    }
}

impl Component for IcalTodo {
    const NAMES: &[&str] = &["VTODO"];
    type Builder = IcalTodoBuilder;
}

impl IcalTodo {
    pub fn get_tzids(&self) -> HashSet<&str> {
        todo!()
    }
}

impl IcalTodoBuilder {
    pub fn get_tzids(&self) -> HashSet<&str> {
        todo!()
    }
}
