use std::collections::HashSet;

use crate::{
    ParserError,
    component::{self},
    parser::{ContentLine, ParserOptions},
};

component::component!(
    IcalAlarm,
    "VALARM",
    required_once: {
    },
    required_many: {
    },
    optional_many: {
    },
    optional_once: {
    },
    components: {},
    extra_fields: {
    },
);

impl IcalAlarmBuilder {
    fn build_from_fields(
        IcalAlarmFields { other_properties }: IcalAlarmFields,
        _: &ParserOptions,
    ) -> Result<IcalAlarm, ParserError> {
        Ok(IcalAlarm { other_properties })
    }
}

impl IcalAlarm {
    pub fn get_tzids(&self) -> HashSet<&str> {
        HashSet::new()
    }
}

impl IcalAlarmBuilder {
    pub fn get_tzids(&self) -> HashSet<&str> {
        HashSet::new()
    }
}
