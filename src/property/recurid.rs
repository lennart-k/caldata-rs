use std::{collections::HashMap, fmt::Write};

use crate::{
    PARAM_DELIMITER, VALUE_DELIMITER,
    generator::{Emitter, LineWriter},
    parser::{ContentLine, ContentLineParams, ICalProperty, ParseProp, ParserError},
    types::{CalDateOrDateTime, Value},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RecurIdRange {
    #[default]
    This,
    ThisAndFuture,
}
#[derive(Debug, Clone)]
pub struct IcalRECURIDProperty(
    pub CalDateOrDateTime,
    pub ContentLineParams,
    pub RecurIdRange,
);
impl ICalProperty for IcalRECURIDProperty {
    const NAME: &'static str = "RECURRENCE-ID";
    const DEFAULT_TYPE: &'static str = "DATE-TIME";

    fn parse_prop(
        prop: &ContentLine,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self, ParserError> {
        let dt = ParseProp::parse_prop(prop, timezones, Self::DEFAULT_TYPE)?;
        let range = match prop.params.get_param("RANGE") {
            Some("THISANDFUTURE") => RecurIdRange::ThisAndFuture,
            None => RecurIdRange::This,
            _ => return Err(ParserError::InvalidPropertyType(prop.generate_string())),
        };
        Ok(Self(dt, prop.params.clone(), range))
    }

    fn utc_or_local(self) -> Self {
        let Self(dt, mut params, range) = self;
        params.remove("TZID");
        Self(dt.utc_or_local(), params, range)
    }
}

impl Emitter for IcalRECURIDProperty {
    fn generate<W: std::fmt::Write>(&self, i: &mut W) -> std::fmt::Result {
        let mut w = LineWriter::new(i);
        w.write_str(Self::NAME)?;
        if !self.1.is_empty() {
            w.write_char(PARAM_DELIMITER)?;
            self.1.generate(&mut w)?;
        }
        w.write_char(VALUE_DELIMITER)?;
        w.write_str(&self.0.value())?;
        w.write_str("\r\n")?;
        Ok(())
    }
}

impl IcalRECURIDProperty {
    pub fn validate_dtstart(&self, dtstart: &CalDateOrDateTime) -> Result<(), ParserError> {
        if (self.0.is_date() != dtstart.is_date())
            || (self.0.timezone().is_local() != dtstart.timezone().is_local())
        {
            return Err(ParserError::DtstartNotMatchingRecurId);
        }
        Ok(())
    }
}

impl From<IcalRECURIDProperty> for crate::property::ContentLine {
    fn from(value: IcalRECURIDProperty) -> Self {
        let mut params = value.1;
        let value_type = value.0.value_type();
        if value_type != IcalRECURIDProperty::DEFAULT_TYPE {
            params.replace_param("VALUE".to_owned(), value_type.to_owned());
        }
        if value.2 == RecurIdRange::ThisAndFuture {
            params.replace_param("RANGE".to_owned(), "THISANDFUTURE".to_owned());
        }
        Self {
            name: IcalRECURIDProperty::NAME.to_owned(),
            params,
            value: value.0.format(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IcalRECURIDProperty;
    use crate::{
        generator::Emitter,
        parser::{ContentLine, ICalProperty},
    };
    use rstest::rstest;

    #[rstest]
    #[case("RECURRENCE-ID;VALUE=DATE:19960401\r\n")]
    #[case("RECURRENCE-ID;RANGE=THISANDFUTURE:19960120T120000Z\r\n")]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::ContentLineParser::from_slice(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let prop = IcalRECURIDProperty::parse_prop(&content_line, None).unwrap();
        similar_asserts::assert_eq!(prop.generate_string(), input);
        let content_line: ContentLine = prop.into();
        similar_asserts::assert_eq!(content_line.generate_string(), input);
    }
}
