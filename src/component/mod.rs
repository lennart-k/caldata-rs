pub mod ical;
pub use ical::{IcalObjectParser, IcalParser, component::*};
pub mod vcard;
pub use vcard::component::*;

use crate::ParserError;
use crate::parser::{ContentLine, ContentLineParser, ParserOptions};
use std::borrow::Cow;
use std::collections::HashMap;

/// An immutable interface for an Ical/Vcard component.
/// This is also implemented by verified components
pub trait Component: Clone {
    const NAMES: &[&str];

    fn get_comp_name(&self) -> &'static str {
        assert_eq!(
            Self::NAMES.len(),
            1,
            "Default implementation only applicable for fixed component name"
        );
        Self::NAMES[0]
    }

    type Unverified: ComponentMut;

    fn get_properties(&self) -> &Vec<ContentLine>;
    fn mutable(self) -> Self::Unverified;

    fn get_property<'c>(&'c self, name: &str) -> Option<&'c ContentLine> {
        self.get_properties().iter().find(|p| p.name == name)
    }

    fn get_named_properties<'c>(&'c self, name: &'c str) -> impl Iterator<Item = &'c ContentLine> {
        self.get_properties().iter().filter(move |p| p.name == name)
    }
}

/// A mutable interface for an Ical/Vcard component.
///
/// It takes a `ContentLineParser` and fills the component with. It's also able to create
/// sub-component used by event and alarms.
pub trait ComponentMut: Component + Default {
    type Verified: Component<Unverified = Self>;

    /// Add the givent sub component.
    fn add_sub_component<'a, T: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        line_parser: &mut ContentLineParser<'a, T>,
        options: &ParserOptions,
    ) -> Result<(), ParserError>;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine>;

    fn remove_property(&mut self, name: &str) {
        self.get_properties_mut().retain(|prop| prop.name != name);
    }

    /// Add the given property.
    #[inline]
    fn add_content_line(&mut self, property: ContentLine) {
        self.get_properties_mut().push(property);
    }

    fn build(
        self,
        options: &ParserOptions,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError>;

    /// Parse the content from `line_parser` and fill the component with.
    fn parse<'a, T: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        line_parser: &mut ContentLineParser<'a, T>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        loop {
            let line = line_parser.next().ok_or(ParserError::NotComplete)??;

            match line.name.as_ref() {
                "END" => break,
                "BEGIN" => self.add_sub_component(&line.value, line_parser, options)?,
                _ => self.add_content_line(line),
            };
        }
        Ok(())
    }

    fn from_parser<'a, T: Iterator<Item = Cow<'a, [u8]>>>(
        line_parser: &mut ContentLineParser<'a, T>,
        options: &ParserOptions,
    ) -> Result<Self, ParserError> {
        let mut out = Self::default();
        out.parse(line_parser, options)?;
        Ok(out)
    }
}
