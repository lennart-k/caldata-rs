use crate::{
    ContentLineParser,
    component::{Component, ComponentMut},
    parser::{ContentLine, ICalProperty, ParserError, ParserOptions},
    property::{GetProperty, IcalDTSTARTProperty, IcalRRULEProperty, IcalTZRDATEProperty},
};
use chrono::{DateTime, Utc};
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
    pub transitions: Vec<IcalTimeZoneTransition>,
}

impl IcalTimeZone {
    pub fn get_tzid(&self) -> &str {
        &self
            .get_property("TZID")
            .expect("we already verified this exists")
            .value
    }

    /// This is a common property containing a timezone identifier from the IANA TZDB
    pub fn get_lic_location(&self) -> Option<&str> {
        self.get_property("X-LIC-LOCATION")
            .map(|prop| prop.value.as_str())
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

    pub fn truncate(self, start: DateTime<Utc>) -> Self {
        Self {
            properties: self.properties,
            transitions: self
                .transitions
                .into_iter()
                .filter_map(|trans| trans.truncate(start))
                .collect(),
        }
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

impl<const VERIFIED: bool> Component for IcalTimeZone<VERIFIED> {
    const NAMES: &[&str] = &["VTIMEZONE"];
    type Unverified = IcalTimeZone<false>;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTimeZone::<false> {
            properties: self.properties,
            transitions: self.transitions,
        }
    }
}

impl ComponentMut for IcalTimeZone<false> {
    type Verified = IcalTimeZone;

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
        if self.get_property("TZID").is_none() {
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

#[derive(Debug, Clone)]
pub struct IcalTimeZoneTransition {
    pub transition: IcalTimeZoneTransitionType,
    pub properties: Vec<ContentLine>,
    pub dtstart: IcalDTSTARTProperty,
}

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZoneTransitionBuilder {
    pub transition: IcalTimeZoneTransitionType,
    pub properties: Vec<ContentLine>,
}

impl IcalTimeZoneTransitionBuilder {
    pub fn new(transition: IcalTimeZoneTransitionType) -> Self {
        Self {
            transition,
            properties: Vec::new(),
        }
    }
}

impl Component for IcalTimeZoneTransition {
    const NAMES: &[&str] = &["STANDARD", "DAYLIGHT"];
    type Unverified = IcalTimeZoneTransitionBuilder;

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
        IcalTimeZoneTransitionBuilder {
            transition: self.transition,
            properties: self.properties,
        }
    }
}

impl Component for IcalTimeZoneTransitionBuilder {
    const NAMES: &[&str] = &["STANDARD", "DAYLIGHT"];
    type Unverified = IcalTimeZoneTransitionBuilder;

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
        self
    }
}

impl ComponentMut for IcalTimeZoneTransitionBuilder {
    type Verified = IcalTimeZoneTransition;

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
        _options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalTimeZoneTransition, ParserError> {
        // Make sure that they are valid
        self.safe_get_all::<IcalRRULEProperty>(None)?;
        self.safe_get_all::<IcalTZRDATEProperty>(None)?;
        Ok(IcalTimeZoneTransition {
            dtstart: self.safe_get_required(None)?,
            transition: self.transition,
            properties: self.properties,
        })
    }
}

impl IcalTimeZoneTransition {
    pub fn truncate(self, start: DateTime<Utc>) -> Option<Self> {
        let dtstart = self.dtstart.0.utc().with_timezone(&crate::rrule::Tz::UTC);
        let mut rrules = vec![];
        let mut rdates = vec![];
        let mut other_properties = vec![];
        let mut dtstart_prop = &ContentLine::default();
        for property in &self.properties {
            match property.name.as_str() {
                "RRULE" => {
                    let rrule = IcalRRULEProperty::parse_prop(property, None)
                        .expect("validated in build")
                        .0;
                    let rrule = rrule.validate(dtstart).ok()?;
                    rrules.push((property, rrule))
                }
                "RDATE" => {
                    let prop = IcalTZRDATEProperty::parse_prop(property, None)
                        .expect("validated in build");
                    if prop.0.is_empty() {
                        continue;
                    }
                    let Some(min_rdate) = prop.0.iter().min().cloned() else {
                        continue;
                    };
                    rdates.push((property, min_rdate));
                }
                "DTSTART" => {
                    dtstart_prop = property;
                }
                _ => other_properties.push(property),
            }
        }

        rrules.retain(|(_content_line, rrule)| {
            if let Some(until) = rrule.get_until()
                && until < &start
            {
                return false;
            }
            true
        });
        rdates.retain(|(_content_line, min_rdate)| min_rdate.utc() >= start);

        if rrules.is_empty() && rdates.is_empty() && dtstart < start {
            return None;
        }

        Some(Self {
            properties: std::iter::once(dtstart_prop.clone())
                .chain(rdates.into_iter().map(|(line, _)| line.clone()))
                .chain(rrules.into_iter().map(|(line, _)| line.clone()))
                .chain(other_properties.into_iter().cloned())
                .collect(),
            transition: self.transition,
            ..self
        })
    }
}

#[cfg(all(test, feature = "vtimezones-rs"))]
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
        assert_snapshot!(format!("{case}_trunc"), tz.generate());
    }

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
