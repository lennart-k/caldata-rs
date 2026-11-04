#[cfg(not(tarpaulin_include))]
use crate::{
    component::component,
    parser::{ContentLine, ParserError},
    property::IcalUIDProperty,
};
use std::collections::HashSet;

component!(
    IcalFreeBusy,
    &["VFREEBUSY"],
    required_once: {
        uid: IcalUIDProperty,
    },
    required_many: {
    },
    optional_many: {
    },
    optional_once: {
    },
    components: {},
    extra_fields: {},
);

impl IcalFreeBusy {
    pub fn get_tzids(&self) -> HashSet<&str> {
        todo!()
    }
}

impl IcalFreeBusyBuilder {
    pub fn get_tzids(&self) -> HashSet<&str> {
        todo!()
    }

    fn build_from_fields(
        IcalFreeBusyFields {
            uid,
            other_properties,
        }: IcalFreeBusyFields,
    ) -> Result<IcalFreeBusy, ParserError> {
        Ok(IcalFreeBusy {
            uid,
            other_properties,
        })
    }
}
