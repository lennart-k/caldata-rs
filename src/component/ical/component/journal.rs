use crate::component::component;
use crate::parser::ParserOptions;
use crate::{
    parser::{ContentLine, ParserError},
    property::{
        IcalDTSTAMPProperty, IcalDTSTARTProperty, IcalEXDATEProperty, IcalEXRULEProperty,
        IcalRDATEProperty, IcalRECURIDProperty, IcalRRULEProperty, IcalUIDProperty,
    },
};

component!(
    IcalJournal,
    "VJOURNAL",
    required_once: {
        uid: IcalUIDProperty,
        dtstamp: IcalDTSTAMPProperty
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
        recurid: IcalRECURIDProperty,
    },
    components: {},
    extra_fields: {
        rruleset: Option<crate::rrule::RRuleSet>
    },
);

impl IcalJournalBuilder {
    fn build_from_fields(
        IcalJournalFields {
            uid,
            dtstamp,
            rdate,
            rrule,
            exdate,
            exrule,
            dtstart,
            recurid,
            other_properties,
        }: IcalJournalFields,
        _: &ParserOptions,
    ) -> Result<IcalJournal, ParserError> {
        Ok(IcalJournal {
            uid,
            dtstamp,
            rdate,
            rrule,
            exdate,
            exrule,
            dtstart,
            recurid,
            other_properties,
            rruleset: None,
        })
    }
}

impl IcalJournal {
    pub fn get_uid(&self) -> &str {
        &self.uid.0
    }

    pub fn get_recurid(&self) -> Option<&IcalRECURIDProperty> {
        self.recurid.as_ref()
    }
}
