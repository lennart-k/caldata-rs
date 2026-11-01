use crate::property::{ParseContentLine, property_enum};
use crate::{
    component::{Component, ComponentMut},
    parser::{ContentLine, ParserError, ParserOptions},
    property::{
        IcalDTSTAMPProperty, IcalDTSTARTProperty, IcalEXDATEProperty, IcalEXRULEProperty,
        IcalRDATEProperty, IcalRECURIDProperty, IcalRRULEProperty, IcalUIDProperty,
    },
};
use std::collections::{HashMap, HashSet};

property_enum!(
    IcalJournalProperty,
    (Uid, IcalUIDProperty),
    (Dtstamp, IcalDTSTAMPProperty),
    (Dtstart, IcalDTSTARTProperty),
    (Rdate, IcalRDATEProperty),
    (Rrule, IcalRRULEProperty),
    (Exdate, IcalEXDATEProperty),
    (Exrule, IcalEXRULEProperty),
    (Recurid, IcalRECURIDProperty)
);

#[derive(Debug, Clone, Default)]
pub struct IcalJournalBuilder {
    pub properties: Vec<IcalJournalProperty>,
}

#[derive(Debug, Clone)]
pub struct IcalJournal {
    properties: Vec<IcalJournalProperty>,
}

impl IcalJournalBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }

    pub fn get_tzids(&self) -> HashSet<&str> {
        todo!()
        // self.properties
        //     .iter()
        //     .filter_map(|prop| prop.params.get_tzid())
        //     .collect()
    }
}

impl IcalJournal {
    pub fn get_uid(&self) -> &str {
        self.properties
            .iter()
            .find_map(|prop| {
                if let IcalJournalProperty::Uid(uid) = prop {
                    Some(uid.0.as_str())
                } else {
                    None
                }
            })
            .expect("must have UID")
    }

    pub fn has_rruleset(&self) -> bool {
        self.properties
            .iter()
            .find(|prop| match prop {
                IcalJournalProperty::Rrule(_)
                | IcalJournalProperty::Rdate(_)
                | IcalJournalProperty::Exrule(_)
                | IcalJournalProperty::Exdate(_) => true,
                _ => false,
            })
            .is_some()
    }
}

impl Component for IcalJournalBuilder {
    const NAMES: &[&str] = &["VJOURNAL"];
    type Builder = IcalJournalBuilder;
}

impl Component for IcalJournal {
    const NAMES: &[&str] = &["VJOURNAL"];
    type Builder = IcalJournalBuilder;
}

impl ComponentMut for IcalJournalBuilder {
    type Verified = IcalJournal;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        let prop = ParseContentLine::from_content_line(content_line, None)?;
        self.properties.push(prop);
        Ok(())
    }

    fn remove_property(&mut self, name: &str) {
        let property_name: IcalJournalPropertyName = name.into();
        self.properties.retain(|prop| prop.name() != property_name);
    }

    fn build(
        self,
        _options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalJournal, ParserError> {
        Ok(IcalJournal {
            properties: self.properties,
        })
    }
}

impl IcalJournal {
    pub fn get_tzids(&self) -> HashSet<&str> {
        HashSet::new()
        // self.properties
        //     .iter()
        //     .filter_map(|prop| prop.params.get_tzid())
        //     .collect()
    }
}
