use crate::{
    IcalTimeZone, IcalTimeZoneBuilder, ParserError,
    component::{Component, ComponentMut},
    generator::Emitter,
    parser::{ContentLine, ICalProperty, ParserOptions},
    property::{IcalCALSCALEProperty, IcalPRODIDProperty, IcalVERSIONProperty},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IcalTimeZoneObject {
    pub version: IcalVERSIONProperty,
    pub prodid: IcalPRODIDProperty,
    pub calscale: Option<IcalCALSCALEProperty>,
    pub other_properties: Vec<ContentLine>,
    pub timezone: IcalTimeZone,
}

impl Component for IcalTimeZoneObject {
    const NAME: &str = "VCALENDAR";

    type Builder = IcalTimeZoneObjectBuilder;

    fn get_tzids(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZoneObjectBuilder {
    pub content_lines: Vec<ContentLine>,
    pub timezone: Option<IcalTimeZone>,
}

impl Component for IcalTimeZoneObjectBuilder {
    const NAME: &str = "VCALENDAR";

    type Builder = Self;

    fn get_tzids(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }
}

impl ComponentMut for IcalTimeZoneObjectBuilder {
    type Verified = IcalTimeZoneObject;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        self.content_lines.push(content_line);
        Ok(())
    }

    fn add_sub_component<'a, T: Iterator<Item = std::borrow::Cow<'a, [u8]>>>(
        &mut self,
        name: &str,
        line_parser: &mut crate::ContentLineParser<'a, T>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        match name {
            IcalTimeZone::NAME if self.timezone.is_none() => {
                self.timezone = Some(
                    IcalTimeZoneBuilder::from_parser(line_parser, options)?.build(options, None)?,
                );
            }
            _ => return Err(ParserError::InvalidComponent(name.to_owned())),
        }
        Ok(())
    }

    fn build(
        self,
        _options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        let mut version: Option<IcalVERSIONProperty> = None;
        let mut calscale: Option<IcalCALSCALEProperty> = None;
        let mut prodid: Option<IcalPRODIDProperty> = None;
        let mut other_properties = Vec::new();

        for content_line in self.content_lines {
            let property_name = content_line.name.as_str();
            match property_name {
                IcalPRODIDProperty::NAME => {
                    prodid = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalCALSCALEProperty::NAME => {
                    calscale = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalVERSIONProperty::NAME => {
                    version = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                _ => other_properties.push(content_line),
            }
        }

        Ok(IcalTimeZoneObject {
            prodid: prodid.ok_or(ParserError::MissingProperty(IcalPRODIDProperty::NAME))?,
            calscale,
            version: version.ok_or(ParserError::MissingProperty(IcalVERSIONProperty::NAME))?,
            other_properties,
            timezone: self.timezone.ok_or(ParserError::NotComplete)?,
        })
    }
}

impl Emitter for IcalTimeZoneObject {
    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        w.write_str("BEGIN:")?;
        w.write_str(Self::NAME)?;
        w.write_str("\r\n")?;
        self.version.generate(w)?;
        self.prodid.generate(w)?;
        self.calscale.generate(w)?;
        for property in &self.other_properties {
            property.generate(w)?;
        }
        self.timezone.generate(w)?;
        w.write_str("END:")?;
        w.write_str(Self::NAME)?;
        w.write_str("\r\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::IcalTimeZoneObjectParser;

    const EXAMPLE_TIMEZONE: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:TEST
CALSCALE:GREGORIAN
BEGIN:VTIMEZONE
TZID:Europe/Berlin
BEGIN:DAYLIGHT
TZOFFSETFROM:+0100
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
DTSTART:19810329T020000
TZNAME:CEST
TZOFFSETTO:+0200
END:DAYLIGHT
BEGIN:STANDARD
TZOFFSETFROM:+0200
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
DTSTART:19961027T030000
TZNAME:CET
TZOFFSETTO:+0100
END:STANDARD
END:VTIMEZONE
END:VCALENDAR
"#;

    #[test]
    fn test_parse_timezone() {
        let timezone = IcalTimeZoneObjectParser::from_slice(EXAMPLE_TIMEZONE.as_bytes())
            .expect_one()
            .unwrap()
            .timezone;
        assert_eq!(timezone.get_tzid(), "Europe/Berlin");
    }
}
