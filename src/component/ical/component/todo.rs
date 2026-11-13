use chrono::DateTime;

use crate::component;
use crate::parser::ParserOptions;
use crate::property::{
    IcalDTSTAMPProperty, IcalDTSTARTProperty, IcalDUEProperty, IcalDURATIONProperty,
};
use crate::rrule::RRuleSet;
use crate::types::Tz;
use crate::{
    component::{Component, IcalAlarm},
    parser::{ContentLine, ParserError},
    property::{
        IcalEXDATEProperty, IcalEXRULEProperty, IcalRDATEProperty, IcalRECURIDProperty,
        IcalRRULEProperty, IcalUIDProperty,
    },
};

component::component!(
    IcalTodo,
    "VTODO",
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
        rruleset: Option<crate::rrule::RRuleSet>
    },
);

impl IcalTodoBuilder {
    fn build_from_fields(
        IcalTodoFields {
            uid,
            dtstamp,
            rdate,
            rrule,
            exdate,
            exrule,
            dtstart,
            duration,
            due,
            recurid,
            other_properties,
            alarms,
        }: IcalTodoFields,
        _: &ParserOptions,
    ) -> Result<IcalTodo, ParserError> {
        let rruleset = if let Some(dtstart) = dtstart.as_ref()
            && !(rrule.is_empty() && rdate.is_empty())
        {
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

        Ok(IcalTodo {
            uid,
            dtstamp,
            rdate,
            rrule,
            exdate,
            exrule,
            dtstart,
            duration,
            due,
            recurid,
            other_properties,
            alarms,
            rruleset,
        })
    }
}

impl IcalTodo {
    pub fn get_uid(&self) -> &str {
        &self.uid.0
    }

    pub fn get_recurid(&self) -> Option<&IcalRECURIDProperty> {
        self.recurid.as_ref()
    }
}
