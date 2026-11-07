use crate::{
    ContentLineParser,
    component::{Component, ComponentMut, IcalTimeZoneTransition, IcalTimeZoneTransitionBuilder},
    generator::Emitter,
    parser::{ContentLine, ICalProperty, ParserError, ParserOptions},
};
use chrono::{DateTime, Utc};
#[cfg(not(tarpaulin_include))]
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

// Memoise generated vtimezones
static TIMEZONES_CACHE: OnceLock<HashMap<String, OnceLock<IcalTimeZone>>> = OnceLock::new();

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZone<const VERIFIED: bool = true> {
    pub other_properties: Vec<ContentLine>,
    pub transitions: Vec<IcalTimeZoneTransition>,
}

impl IcalTimeZone {
    pub fn get_tzid(&self) -> &str {
        todo!()
    }

    /// This is a common property containing a timezone identifier from the IANA TZDB
    pub fn get_lic_location(&self) -> Option<&str> {
        todo!()
    }

    pub fn from_tzid(tzid: &str) -> Option<&Self> {
        let timezones = TIMEZONES_CACHE.get_or_init(|| {
            let mut timezones = HashMap::new();
            for tzid in vtimezones_rs::VTIMEZONES.keys() {
                timezones
                    .entry(tzid.to_string())
                    .or_insert_with(OnceLock::new);
            }
            timezones
        });

        let lock = timezones.get(tzid)?;
        Some(lock.get_or_init(|| {
            use crate::IcalParser;

            let tz_ics = *vtimezones_rs::VTIMEZONES.get(tzid).unwrap();
            let cal = IcalParser::from_slice(tz_ics.as_bytes())
                .expect_one()
                .unwrap();
            cal.vtimezones.into_values().next().unwrap()
        }))
    }

    pub fn truncate(self, start: DateTime<Utc>) -> Self {
        Self {
            other_properties: self.other_properties,
            transitions: self
                .transitions
                .into_iter()
                .filter_map(|trans| trans.truncate(start))
                .collect(),
        }
    }
}

impl From<&IcalTimeZone> for Option<chrono_tz::Tz> {
    fn from(value: &IcalTimeZone) -> Self {
        use crate::types::get_proprietary_tzid;
        use std::str::FromStr;

        // Try X-LIC-LOCATION
        if let Some(loc) = value.get_lic_location()
            && let Ok(tz) = chrono_tz::Tz::from_str(loc)
        {
            return Some(tz);
        };

        // Try using TZID in Olson DB
        let tzid = value.get_tzid();
        if let Ok(tz) = chrono_tz::Tz::from_str(tzid) {
            return Some(tz);
        }
        // Try map of proprietary timezone IDs (mostly for Microsoft products)
        get_proprietary_tzid(tzid)
    }
}

impl<const VERIFIED: bool> Component for IcalTimeZone<VERIFIED> {
    const NAMES: &[&str] = &["VTIMEZONE"];
    type Builder = IcalTimeZone<false>;
}

impl ComponentMut for IcalTimeZone<false> {
    type Verified = IcalTimeZone;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        self.other_properties.push(content_line);
        Ok(())
    }

    fn add_sub_component<'a, I: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        line_parser: &mut ContentLineParser<'a, I>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        use self::IcalTimeZoneTransitionType::{DAYLIGHT, STANDARD};

        match value {
            "STANDARD" => {
                let mut transition = IcalTimeZoneTransitionBuilder::new(STANDARD);
                transition.parse(line_parser, options)?;
                self.transitions.push(transition.build(options, None)?);
            }
            "DAYLIGHT" => {
                let mut transition = IcalTimeZoneTransitionBuilder::new(DAYLIGHT);
                transition.parse(line_parser, options)?;
                self.transitions.push(transition.build(options, None)?);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        _options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalTimeZone, ParserError> {
        todo!();
        // if self.get_property("TZID").is_none() {
        //     return Err(ParserError::MissingProperty("TZID"));
        // }

        let verified = IcalTimeZone {
            other_properties: self.other_properties,
            transitions: self.transitions,
        };

        #[cfg(feature = "test")]
        {
            // Verify that the conditions for our getters are actually met
            verified.get_tzid();
            verified.get_lic_location();
        }

        Ok(verified)
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum IcalTimeZoneTransitionType {
    #[default]
    STANDARD,
    DAYLIGHT,
}

impl Emitter for IcalTimeZone {
    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use insta::assert_snapshot;
    use rstest::rstest;

    use crate::{component::IcalTimeZone, generator::Emitter};

    #[rstest]
    #[case(0, "Europe/Bratislava")]
    #[case(1, "Europe/Berlin")]
    fn test_truncation(#[case] case: usize, #[case] tzid: &str) {
        let tz = IcalTimeZone::from_tzid(tzid)
            .unwrap()
            .clone()
            .truncate(Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap());
        assert_snapshot!(format!("{case}_trunc"), tz.generate_string());
    }

    #[rstest]
    #[case("Europe/Berlin")]
    #[case("CET")]
    fn test_timezone(#[case] tzid: &str) {
        let tz = IcalTimeZone::from_tzid(tzid).unwrap();
        assert_eq!(tz.get_tzid(), tzid);
        assert!(tz.generate_string().contains(tzid));
    }

    #[test]
    fn test_all_timezones() {
        for tzid in vtimezones_rs::VTIMEZONES.keys() {
            let tz = IcalTimeZone::from_tzid(tzid).unwrap();
            assert_eq!(&tz.get_tzid(), tzid);
            assert!(tz.generate_string().contains(tzid));
        }
    }
}
