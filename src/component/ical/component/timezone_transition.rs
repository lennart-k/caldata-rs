use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{
    ParserError,
    component::{Component, ComponentMut, IcalTimeZoneTransitionType},
    parser::{ContentLine, ICalProperty, ParserOptions},
    property::{IcalDTSTARTProperty, IcalRDATEProperty, IcalRRULEProperty, IcalTZRDATEProperty},
    types::Tz,
};

#[derive(Debug, Clone)]
pub struct IcalTimeZoneTransition {
    pub transition: IcalTimeZoneTransitionType,
    pub dtstart: IcalDTSTARTProperty,
    pub rrule: Vec<IcalRRULEProperty>,
    pub rdate: Vec<IcalRDATEProperty>,
    pub other_properties: Vec<ContentLine>,
}

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZoneTransitionBuilder {
    pub transition: IcalTimeZoneTransitionType,
    pub other_properties: Vec<ContentLine>,
}

impl IcalTimeZoneTransitionBuilder {
    pub fn new(transition: IcalTimeZoneTransitionType) -> Self {
        Self {
            transition,
            other_properties: Vec::new(),
        }
    }
}

impl Component for IcalTimeZoneTransition {
    const NAMES: &[&str] = &["STANDARD", "DAYLIGHT"];
    type Builder = IcalTimeZoneTransitionBuilder;

    fn get_comp_name(&self) -> &'static str {
        match self.transition {
            IcalTimeZoneTransitionType::STANDARD => "STANDARD",
            IcalTimeZoneTransitionType::DAYLIGHT => "DAYLIGHT",
        }
    }
}

impl Component for IcalTimeZoneTransitionBuilder {
    const NAMES: &[&str] = &["STANDARD", "DAYLIGHT"];
    type Builder = IcalTimeZoneTransitionBuilder;

    fn get_comp_name(&self) -> &'static str {
        match self.transition {
            IcalTimeZoneTransitionType::STANDARD => "STANDARD",
            IcalTimeZoneTransitionType::DAYLIGHT => "DAYLIGHT",
        }
    }
}

impl ComponentMut for IcalTimeZoneTransitionBuilder {
    type Verified = IcalTimeZoneTransition;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        self.other_properties.push(content_line);
        Ok(())
    }

    fn build(
        self,
        _options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalTimeZoneTransition, ParserError> {
        // Make sure that they are valid
        let mut rrule = Vec::new();
        let mut rdate = Vec::new();
        let mut dtstart = None;
        let mut other_properties = Vec::new();
        for content_line in self.other_properties {
            match content_line.name.as_str() {
                IcalRRULEProperty::NAME => {
                    rrule.push(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalRDATEProperty::NAME => {
                    rdate.push(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalDTSTARTProperty::NAME => {
                    dtstart = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                _ => other_properties.push(content_line),
            };
        }
        Ok(IcalTimeZoneTransition {
            dtstart: dtstart.unwrap(),
            rrule,
            rdate,
            transition: self.transition,
            other_properties,
        })
    }
}

impl IcalTimeZoneTransition {
    pub fn truncate(self, start: DateTime<Utc>) -> Option<Self> {
        todo!()
    }
}
