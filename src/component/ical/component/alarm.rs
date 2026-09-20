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
}

impl Component for IcalAlarm {
    const NAMES: &[&str] = &["VALARM"];
    type Builder = IcalAlarmBuilder;
}

impl ComponentMut for IcalAlarmBuilder {
    type Verified = IcalAlarm;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        self.properties.push(content_line);
        Ok(())
    }

    fn remove_property(&mut self, name: &str) {
        self.properties
            .retain(|content_line| content_line.name != name);
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
