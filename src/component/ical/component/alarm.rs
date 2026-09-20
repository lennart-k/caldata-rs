#[cfg(not(tarpaulin_include))]
use crate::parser::ParserOptions;
use crate::{
    component::{Component, ComponentMut},
    parser::{ContentLine, ParserError},
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct IcalAlarmBuilder {
    pub properties: Vec<ContentLine>,
}

#[derive(Debug, Clone)]
pub struct IcalAlarm {
    pub properties: Vec<ContentLine>,
}

impl IcalAlarmBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl Component for IcalAlarmBuilder {
    const NAMES: &[&str] = &["VALARM"];
    type Builder = IcalAlarmBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Builder {
        self
    }
}

impl Component for IcalAlarm {
    const NAMES: &[&str] = &["VALARM"];
    type Builder = IcalAlarmBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Builder {
        IcalAlarmBuilder {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalAlarmBuilder {
    type Verified = IcalAlarm;

    fn add_content_line(&mut self, content_line: ContentLine) {
        self.properties.push(content_line);
    }

    fn build(
        self,
        _options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalAlarm, ParserError> {
        Ok(IcalAlarm {
            properties: self.properties,
        })
    }
}

impl IcalAlarm {
    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .collect()
    }
}

impl IcalAlarmBuilder {
    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .collect()
    }
}
