use crate::{
    ContentLineParser,
    component::{Component, ComponentMut},
    parser::{ContentLine, ParserError, ParserOptions},
};
#[cfg(not(tarpaulin_include))]
use std::borrow::Cow;
use std::collections::HashMap;
#[cfg(feature = "vtimezones-rs")]
use std::sync::OnceLock;

// Memoise generated vtimezones
#[cfg(feature = "vtimezones-rs")]
static TIMEZONES_CACHE: OnceLock<HashMap<String, OnceLock<IcalTimeZone>>> = OnceLock::new();

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZone<const VERIFIED: bool = true> {
    pub properties: Vec<ContentLine>,
    pub transitions: Vec<IcalTimeZoneTransition<true>>,
}

impl IcalTimeZone {
    pub fn get_tzid(&self) -> &str {
        self.get_property("TZID")
            .and_then(|prop| prop.value.as_ref())
            .expect("we already verified this exists")
    }

    /// This is a common property containing a timezone identifier from the IANA TZDB
    pub fn get_lic_location(&self) -> Option<&str> {
        self.get_property("X-LIC-LOCATION")
            .and_then(|prop| prop.value.as_deref())
    }

    #[cfg(feature = "vtimezones-rs")]
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
}

#[cfg(feature = "chrono-tz")]
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

impl IcalTimeZone<false> {
    pub fn new() -> IcalTimeZone<false> {
        IcalTimeZone {
            properties: Vec::new(),
            transitions: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalTimeZone<VERIFIED> {
    const NAMES: &[&str] = &["VTIMEZONE"];
    type Unverified = IcalTimeZone<false>;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTimeZone {
            properties: self.properties,
            transitions: self.transitions,
        }
    }
}

impl ComponentMut for IcalTimeZone<false> {
    type Verified = IcalTimeZone<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
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
                let mut transition = IcalTimeZoneTransition::new(STANDARD);
                transition.parse(line_parser, options)?;
                self.transitions.push(transition.build(None)?);
            }
            "DAYLIGHT" => {
                let mut transition = IcalTimeZoneTransition::new(DAYLIGHT);
                transition.parse(line_parser, options)?;
                self.transitions.push(transition.build(None)?);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalTimeZone<true>, ParserError> {
        if !matches!(
            self.get_property("TZID"),
            Some(&ContentLine { value: Some(_), .. }),
        ) {
            return Err(ParserError::MissingProperty("TZID"));
        }

        let verified = IcalTimeZone {
            properties: self.properties,
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

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZoneTransition<const VERIFIED: bool = true> {
    pub transition: IcalTimeZoneTransitionType,
    pub properties: Vec<ContentLine>,
}

impl IcalTimeZoneTransition<false> {
    pub fn new(transition: IcalTimeZoneTransitionType) -> Self {
        Self {
            transition,
            properties: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalTimeZoneTransition<VERIFIED> {
    const NAMES: &[&str] = &["STANDARD", "DAYLIGHT"];
    type Unverified = IcalTimeZoneTransition<false>;

    fn get_comp_name(&self) -> &'static str {
        match self.transition {
            IcalTimeZoneTransitionType::STANDARD => "STANDARD",
            IcalTimeZoneTransitionType::DAYLIGHT => "DAYLIGHT",
        }
    }

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTimeZoneTransition {
            transition: self.transition,
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalTimeZoneTransition<false> {
    type Verified = IcalTimeZoneTransition<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    #[cfg(not(tarpaulin_include))]
    fn add_sub_component<'a, I: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        _: &mut ContentLineParser<'a, I>,
        _options: &ParserOptions,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent(value.to_owned()))
    }

    fn build(
        self,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalTimeZoneTransition<true>, ParserError> {
        Ok(IcalTimeZoneTransition {
            transition: self.transition,
            properties: self.properties,
        })
    }
}

#[cfg(all(test, feature = "vtimezones-rs"))]
mod tests {
    use rstest::rstest;

    use crate::{component::IcalTimeZone, generator::Emitter};

    #[rstest]
    #[case("Europe/Berlin")]
    #[case("CET")]
    fn test_timezone(#[case] tzid: &str) {
        let tz = IcalTimeZone::from_tzid(tzid).unwrap();
        assert_eq!(tz.get_tzid(), tzid);
        assert!(tz.generate().contains(tzid));
    }

    #[test]
    fn test_all_timezones() {
        for tzid in vtimezones_rs::VTIMEZONES.keys() {
            let tz = IcalTimeZone::from_tzid(tzid).unwrap();
            assert_eq!(&tz.get_tzid(), tzid);
            assert!(tz.generate().contains(tzid));
        }
    }
}
