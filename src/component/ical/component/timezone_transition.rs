use crate::{
    ParserError,
    component::{Component, ComponentMut},
    generator::Emitter,
    parser::{ContentLine, ICalProperty, ParserOptions},
    property::{
        IcalDTSTARTProperty, IcalRDATEProperty, IcalRRULEProperty, IcalTZNAMEProperty,
        IcalTZOFFSETFROMProperty, IcalTZOFFSETTOProperty,
    },
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IcalTimeZoneTransition<const DAYLIGHT: bool> {
    pub dtstart: IcalDTSTARTProperty,
    pub tzoffsetfrom: IcalTZOFFSETFROMProperty,
    pub tzoffsetto: IcalTZOFFSETTOProperty,
    pub rrule: Vec<IcalRRULEProperty>,
    pub rdate: Vec<IcalRDATEProperty>,
    pub tzname: Option<IcalTZNAMEProperty>,
    pub other_properties: Vec<ContentLine>,
}

pub type IcalTimeZoneTransitionStandard = IcalTimeZoneTransition<false>;
pub type IcalTimeZoneTransitionDaylight = IcalTimeZoneTransition<true>;

impl<const DAYLIGHT: bool> Component for IcalTimeZoneTransition<DAYLIGHT> {
    const NAME: &str = if DAYLIGHT { "DAYLIGHT" } else { "STANDARD" };
    type Builder = IcalTimeZoneTransitionBuilder<DAYLIGHT>;
}

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZoneTransitionBuilder<const DAYLIGHT: bool> {
    pub other_properties: Vec<ContentLine>,
}

pub type IcalTimeZoneTransitionStandardBuilder = IcalTimeZoneTransitionBuilder<false>;
pub type IcalTimeZoneTransitionDaylightBuilder = IcalTimeZoneTransitionBuilder<true>;

impl<const DAYLIGHT: bool> Component for IcalTimeZoneTransitionBuilder<DAYLIGHT> {
    const NAME: &str = if DAYLIGHT { "DAYLIGHT" } else { "STANDARD" };
    type Builder = IcalTimeZoneTransitionBuilder<DAYLIGHT>;
}

impl<const DAYLIGHT: bool> ComponentMut for IcalTimeZoneTransitionBuilder<DAYLIGHT> {
    type Verified = IcalTimeZoneTransition<DAYLIGHT>;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        self.other_properties.push(content_line);
        Ok(())
    }

    fn build(
        self,
        _options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        // Make sure that they are valid
        let mut rrule = Vec::new();
        let mut rdate = Vec::new();
        let mut dtstart = None;
        let mut tzoffsetfrom = None;
        let mut tzoffsetto = None;
        let mut tzname = None;
        let mut other_properties = Vec::new();
        for content_line in self.other_properties {
            match content_line.name.as_str() {
                IcalRRULEProperty::NAME => {
                    rrule.push(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalRDATEProperty::NAME => {
                    rdate.push(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalDTSTARTProperty::NAME => {
                    dtstart = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalTZOFFSETFROMProperty::NAME => {
                    tzoffsetfrom = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalTZOFFSETTOProperty::NAME => {
                    tzoffsetto = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalTZNAMEProperty::NAME => {
                    tzname = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                _ => other_properties.push(content_line),
            };
        }
        Ok(IcalTimeZoneTransition {
            dtstart: dtstart.ok_or(ParserError::MissingProperty(IcalDTSTARTProperty::NAME))?,
            tzoffsetfrom: tzoffsetfrom
                .ok_or(ParserError::MissingProperty(IcalTZOFFSETFROMProperty::NAME))?,
            tzoffsetto: tzoffsetto
                .ok_or(ParserError::MissingProperty(IcalTZOFFSETTOProperty::NAME))?,
            rrule,
            rdate,
            tzname,
            other_properties,
        })
    }
}

impl<const DAYLIGHT: bool> Emitter for IcalTimeZoneTransition<DAYLIGHT> {
    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        w.write_str("BEGIN:")?;
        w.write_str(Self::NAME)?;
        w.write_str("\r\n")?;
        self.dtstart.generate(w)?;
        self.tzoffsetfrom.generate(w)?;
        self.tzoffsetto.generate(w)?;
        self.rrule.generate(w)?;
        self.rdate.generate(w)?;
        self.tzname.generate(w)?;
        for property in &self.other_properties {
            property.generate(w)?;
        }
        w.write_str("END:")?;
        w.write_str(Self::NAME)?;
        w.write_str("\r\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use chrono::Duration;

    use crate::{
        ComponentParser,
        component::{IcalTimeZoneTransitionDaylight, IcalTimeZoneTransitionStandard},
        generator::Emitter,
        types::{CalDateTime, UtcOffset},
    };

    const EXAMPLE_DAYLIGHT: &str = r#"BEGIN:DAYLIGHT
DTSTART:19760425T020000
TZOFFSETFROM:-0500
TZOFFSETTO:-0400
RRULE:FREQ=YEARLY;UNTIL=19860427T070000Z;BYMONTH=4;BYDAY=-1SU
TZNAME:ET
COMMENT:HELLO
END:DAYLIGHT
"#;

    const EXAMPLE_STANDARD: &str = r#"BEGIN:STANDARD
DTSTART:19760425T020000
TZOFFSETFROM:-0500
TZOFFSETTO:-0400
RRULE:FREQ=YEARLY;UNTIL=19860427T070000Z;BYMONTH=4;BYDAY=-1SU
TZNAME:ET
COMMENT:HELLO
END:STANDARD
"#;

    #[test]
    fn test_parse_timezone_transitions() {
        let transition = ComponentParser::<'_, IcalTimeZoneTransitionDaylight, _>::from_slice(
            EXAMPLE_DAYLIGHT.as_bytes(),
        )
        .expect_one()
        .unwrap();
        assert_eq!(
            transition.tzoffsetfrom.0,
            UtcOffset(Duration::new(-5 * 3600, 0).unwrap())
        );
        assert_eq!(
            transition.tzoffsetto.0,
            UtcOffset(Duration::new(-4 * 3600, 0).unwrap())
        );
        assert_matches!(
            transition.dtstart.0,
            crate::types::CalDateOrDateTime::DateTime(CalDateTime(_))
        );
    }

    #[test]
    fn test_generate_daylight() {
        let transition = ComponentParser::<'_, IcalTimeZoneTransitionDaylight, _>::from_slice(
            EXAMPLE_DAYLIGHT.as_bytes(),
        )
        .expect_one()
        .unwrap();
        similar_asserts::assert_eq!(
            EXAMPLE_DAYLIGHT.replace("\n", "\r\n"),
            transition.generate_string()
        );
    }

    #[test]
    fn test_generate_standard() {
        let transition = ComponentParser::<'_, IcalTimeZoneTransitionStandard, _>::from_slice(
            EXAMPLE_STANDARD.as_bytes(),
        )
        .expect_one()
        .unwrap();
        similar_asserts::assert_eq!(
            EXAMPLE_STANDARD.replace("\n", "\r\n"),
            transition.generate_string()
        );
    }
}
