use crate::{
    ParserError,
    component::{self, Component, IcalTimeZoneTransitionDaylight, IcalTimeZoneTransitionStandard},
    parser::{ContentLine, ParserOptions},
    property::{IcalTZIDProperty, IcalXLICLOCATIONProperty},
};
use chrono::{DateTime, Utc};
use std::{collections::HashMap, sync::OnceLock};

// Memoise generated vtimezones
static TIMEZONES_CACHE: OnceLock<HashMap<String, OnceLock<IcalTimeZone>>> = OnceLock::new();

component::component!(
    IcalTimeZone,
    "VTIMEZONE",
    required_once: {
        tzid: IcalTZIDProperty,
    },
    required_many: {
    },
    optional_many: {
    },
    optional_once: {
        lic_location: IcalXLICLOCATIONProperty
    },
    components: {
        standard: IcalTimeZoneTransitionStandard,
        daylight: IcalTimeZoneTransitionDaylight,
    },
    extra_fields: {
    },
);

impl IcalTimeZoneBuilder {
    fn build_from_fields(
        IcalTimeZoneFields {
            tzid,
            lic_location,
            other_properties,
            standard,
            daylight,
        }: IcalTimeZoneFields,
        _: &ParserOptions,
    ) -> Result<IcalTimeZone, ParserError> {
        Ok(IcalTimeZone {
            tzid,
            lic_location,
            other_properties,
            standard,
            daylight,
        })
    }
}

impl IcalTimeZone {
    pub fn get_tzid(&self) -> &str {
        &self.tzid.0
    }

    pub fn get_lic_location(&self) -> Option<&str> {
        self.lic_location.as_ref().map(|prop| prop.0.as_str())
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
            cal.get_timezones().first().unwrap().clone()
        }))
    }

    pub fn truncate(self, _start: DateTime<Utc>) -> Self {
        // TODO: implement
        self
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{ComponentParser, component::IcalTimeZone, generator::Emitter};

    const EXAMPLE_TIMEZONE: &str = r#"BEGIN:VTIMEZONE
TZID:America/New_York
LAST-MODIFIED:20050809T050000Z
BEGIN:STANDARD
DTSTART:20071104T020000
TZOFFSETFROM:-0400
TZOFFSETTO:-0500
TZNAME:EST
END:STANDARD
BEGIN:DAYLIGHT
DTSTART:20070311T020000
TZOFFSETFROM:-0500
TZOFFSETTO:-0400
TZNAME:EDT
END:DAYLIGHT
END:VTIMEZONE
"#;

    #[test]
    fn test_parse_timezone() {
        let timezone =
            ComponentParser::<'_, IcalTimeZone, _>::from_slice(EXAMPLE_TIMEZONE.as_bytes())
                .expect_one()
                .unwrap();
        assert_eq!(timezone.tzid.0, "America/New_York");
        assert_eq!(timezone.standard.len(), 1);
        assert_eq!(
            timezone
                .standard
                .first()
                .unwrap()
                .tzname
                .as_ref()
                .unwrap()
                .0,
            "EST"
        );
        assert_eq!(timezone.daylight.len(), 1);
        assert_eq!(
            timezone
                .daylight
                .first()
                .unwrap()
                .tzname
                .as_ref()
                .unwrap()
                .0,
            "EDT"
        );
    }

    #[test]
    fn test_generate_daylight() {
        let timezone =
            ComponentParser::<'_, IcalTimeZone, _>::from_slice(EXAMPLE_TIMEZONE.as_bytes())
                .expect_one()
                .unwrap();
        similar_asserts::assert_eq!(
            EXAMPLE_TIMEZONE.replace("\n", "\r\n"),
            timezone.generate_string()
        );
    }

    #[rstest::rstest]
    #[case("Europe/Berlin")]
    #[case("America/New_York")]
    #[case("Europe/Kyiv")]
    #[case("CET")]
    fn test_tzdb(#[case] tzid: &str) {
        let timezone = IcalTimeZone::from_tzid(tzid).unwrap();
        assert_eq!(timezone.tzid.0, tzid);
        let tz: Option<chrono_tz::Tz> = timezone.into();
        assert_eq!(chrono_tz::Tz::from_str(tzid).unwrap(), tz.unwrap());
    }

    #[rstest::rstest]
    #[case("asdjlk")]
    fn test_tzdb_invalid(#[case] tzid: &str) {
        assert!(IcalTimeZone::from_tzid(tzid).is_none());
    }
}
