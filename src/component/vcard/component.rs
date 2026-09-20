use crate::component::{Component, ComponentMut};
use crate::parser::{ContentLine, ParserError, ParserOptions};
use crate::property::{
    IcalUIDProperty, VcardANNIVERSARYProperty, VcardBDAYProperty, VcardFNProperty, VcardNProperty,
};
use std::collections::HashMap;

macro_rules! vcard_property {
    ($name:ident, $(($prop_variant:ident, $prop_name:literal, $prop_type:ty)),*) => {
        ::paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum [<$name Name>] {
                $($prop_variant),*,
                Other(String),
            }

            impl From<&str> for [<$name Name>] {
                fn from(name: &str) -> Self {
                    match name {
                        $($prop_name => Self::$prop_variant),*,
                        other => Self::Other(other.to_string())
                    }
                }
            }

            #[derive(Debug, Clone)]
            pub enum $name {
                $($prop_variant($prop_type)),*,
                Other(crate::parser::ContentLine)
            }

            impl $name {
                pub fn name(&self) -> [<$name Name>] {
                    match self {
                        $(Self::$prop_variant(_) => [<$name Name>]::$prop_variant),*,
                        Self::Other(content_line) => [<$name Name>]::Other(content_line.name.clone())
                    }
                }
            }

            impl ParseContentLine for $name {
                fn from_content_line(
                    content_line: ContentLine,
                    timezones: Option<&std::collections::HashMap<String, Option<chrono_tz::Tz>>>,
                ) -> Result<Self, ParserError> {
                    let property_name: [<$name Name>] = content_line.name.as_str().into();

                    Ok(match property_name {
                        $([<$name Name>]::$prop_variant => $name::$prop_variant(crate::parser::ICalProperty::parse_prop(&content_line, timezones)?)),*,
                        [<$name Name>]::Other(_) => $name::Other(content_line)
                    })
                }
            }
        }
    };
}

pub trait ParseContentLine: Sized {
    fn from_content_line(
        content_line: ContentLine,
        timezones: Option<&std::collections::HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self, ParserError>;
}

vcard_property!(
    VcardProperty,
    (Uid, "UID", IcalUIDProperty),
    (Fn, "FN", VcardFNProperty),
    (N, "N", VcardNProperty),
    (Bday, "BDAY", VcardBDAYProperty),
    (Anniversary, "ANNIVERSARY", VcardANNIVERSARYProperty)
);

#[derive(Debug, Clone)]
pub struct VcardContact {
    pub properties: Vec<VcardProperty>,
}

#[derive(Debug, Clone, Default)]
pub struct VcardContactBuilder {
    pub properties: Vec<VcardProperty>,
}

impl VcardContact {
    pub fn get_uid(&self) -> Option<&str> {
        None
        // self.properties.iter().find(predicate)
    }
}

impl Component for VcardContactBuilder {
    const NAMES: &[&str] = &["VCARD"];
    type Builder = VcardContactBuilder;

    fn mutable(self) -> Self::Builder {
        self
    }
}

impl Component for VcardContact {
    const NAMES: &[&str] = &["VCARD"];
    type Builder = VcardContactBuilder;

    fn mutable(self) -> Self::Builder {
        VcardContactBuilder {
            properties: self.properties,
        }
    }
}

impl ComponentMut for VcardContactBuilder {
    type Verified = VcardContact;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        let prop = ParseContentLine::from_content_line(content_line, None)?;
        self.properties.push(prop);
        Ok(())
    }

    fn remove_property(&mut self, name: &str) {
        let property_name: VcardPropertyName = name.into();
        self.properties.retain(|prop| prop.name() != property_name);
    }

    fn build(
        self,
        _options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        // let uid = self
        //     .safe_get_optional(timezones)?
        //     .map(|IcalUIDProperty(uid, _)| uid);

        // let name = self.safe_get_optional(timezones)?;
        // let full_name = self.safe_get_all(timezones)?;
        // let birthday = self.safe_get_optional(timezones)?;
        // let anniversary = self.safe_get_optional(timezones)?;

        let verified = VcardContact {
            properties: self.properties,
        };

        Ok(verified)
    }
}
