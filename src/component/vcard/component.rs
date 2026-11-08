use crate::component::component;
use crate::parser::{ContentLine, ParserError, ParserOptions};
use crate::property::{
    VcardANNIVERSARYProperty, VcardBDAYProperty, VcardFNProperty, VcardNProperty, VcardUIDProperty,
};

component!(
    VcardContact,
    "VCARD",
    required_once: {
    },
    required_many: {
        full_name: VcardFNProperty,
    },
    optional_many: {

    },
    optional_once: {
        uid: VcardUIDProperty,
        n: VcardNProperty,
        bday: VcardBDAYProperty,
        anniversary: VcardANNIVERSARYProperty,
    },
    components: {},
    extra_fields: {},
);

impl VcardContactBuilder {
    fn build_from_fields(
        VcardContactFields {
            full_name,
            uid,
            n,
            bday,
            anniversary,
            other_properties,
        }: VcardContactFields,
        _: &ParserOptions,
    ) -> Result<VcardContact, ParserError> {
        Ok(VcardContact {
            full_name,
            uid,
            n,
            bday,
            anniversary,
            other_properties,
        })
    }
}

impl VcardContact {
    pub fn get_uid(&self) -> Option<&str> {
        self.uid.as_ref().map(|uid| uid.0.as_str())
    }
}
