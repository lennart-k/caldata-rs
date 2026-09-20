use strum::EnumString;

use crate::component::{Component, ComponentMut};
use crate::parser::{ContentLine, ParserError, ParserOptions};
use crate::property::{
    GetProperty, IcalUIDProperty, VcardANNIVERSARYProperty, VcardBDAYProperty, VcardFNProperty,
    VcardNProperty,
};
use std::collections::HashMap;

#[derive(Debug, Clone, EnumString)]
pub enum VcardPropertyName {
    Uid,
    Fn,
    N,
    Bday,
    Anniversary,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Clone)]
pub enum VcardProperty {
    Uid(String),
    Fn(VcardFNProperty),
    N(VcardNProperty),
    Bday(VcardBDAYProperty),
    Anniversary(VcardANNIVERSARYProperty),
    Other(ContentLine),
}

#[derive(Debug, Clone)]
pub struct VcardContact {
    pub properties: Vec<VcardProperty>,
    // pub uid: Option<String>,
    // pub full_name: Vec<VcardFNProperty>,
    // pub name: Option<VcardNProperty>,
    // pub birthday: Option<VcardBDAYProperty>,
    // pub anniversary: Option<VcardANNIVERSARYProperty>,
    // pub properties: Vec<ContentLine>,
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

    fn add_content_line(&mut self, content_line: ContentLine) {
        self.properties.push(content_line);
    }

    fn remove_property(&mut self, name: &str) {
        self.properties
            .retain(|content_line| content_line.name != name);
    }

    fn build(
        self,
        _options: &ParserOptions,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        let uid = self
            .safe_get_optional(timezones)?
            .map(|IcalUIDProperty(uid, _)| uid);

        // let name = self.safe_get_optional(timezones)?;
        // let full_name = self.safe_get_all(timezones)?;
        // let birthday = self.safe_get_optional(timezones)?;
        // let anniversary = self.safe_get_optional(timezones)?;

        let verified = VcardContact {
            // uid,
            // name,
            // full_name,
            // birthday,
            // anniversary,
            properties: self.properties,
        };

        Ok(verified)
    }
}
