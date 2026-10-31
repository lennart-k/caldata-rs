mod utils;

macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

pub mod calendar_object {
    use caldata::{
        IcalObjectParser, IcalParser, component::CalendarInnerData, generator::Emitter, types::Tz,
    };
    use chrono::{DateTime, Timelike, Utc};
    use itertools::Itertools;

    #[rstest::rstest]
    #[case(0, include_str!("./resources/ical_example_1.ics"), "W. Europe Standard Time")]
    #[case(1, include_str!("./resources/ical_example_2.ics"), "W. Europe Standard Time")]
    #[case(2, include_str!("./resources/ical_example_rrule.ics"), "Europe/Berlin")]
    #[case(3, include_str!("./resources/ical_events.ics"), "")]
    #[case(4, include_str!("./resources/ical_special_symbols.ics"), "")]
    #[case(5, include_str!("./resources/ical_todos.ics"), "")]
    #[case(6, include_str!("./resources/ical_journals.ics"), "")]
    #[case(7, include_str!("./resources/recurring_wholeday.ics"), "")]
    // Calendar objects from Thunderbird are invalid since their VTIMEZONE object RRULEs contain an
    // UNTIL datetime in local time (this MUST be UTC)
    #[case(8, include_str!("./resources/ical_thunderbird.ics"), "Europe/Berlin")]
    #[case(9, include_str!("./resources/ical_recurrence_date.ics"), "")]
    #[case(10, include_str!("./resources/ical_recurrence_date_2.ics"), "")]
    fn valid_objects(#[case] case: usize, #[case] input: &str, #[case] tzids: &str) {
        set_snapshot_suffix!("{case}");
        let generic_reader = IcalParser::from_slice(input.as_bytes());
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        for (g_res, res) in generic_reader.zip(reader) {
            let g_cal = g_res.unwrap();
            let cal = res.unwrap();
            similar_asserts::assert_eq!(g_cal.generate_string(), cal.generate_string());
            similar_asserts::assert_eq!(cal.get_tzids().iter().sorted().join(","), tzids);
        }
    }

    #[rstest::rstest]
    #[case(0, include_str!("./resources/ical_freebusy.ics"))]
    fn invalid_objects(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        for res in reader {
            assert!(res.is_err());
        }
    }

    /// Expand a reccurent event across winter and summer time
    /// This will fail if we wrongly expand in UTC
    #[rstest::rstest]
    fn rrule_expansion_timezones() {
        let input = include_str!("./resources/ical_recurrence_winter_summer.ics");
        let obj = IcalObjectParser::from_slice(input.as_bytes())
            .expect_one()
            .unwrap();
        let CalendarInnerData::Event(event, _) = obj.get_inner() else {
            panic!()
        };
        let expanded = event.expand_recurrence(None, None, &[]);
        for recurrence in expanded {
            let datetime: DateTime<Tz> = recurrence.dtstart.0.clone().into();
            let datetime_local = datetime.with_timezone(&Tz::Olson(chrono_tz::Tz::Europe__Berlin));
            assert_eq!(datetime_local.hour(), 9);
        }
    }

    #[rstest::rstest]
    #[case(0, include_str!("./resources/Recurring at 9am, third at 10am.ics"))]
    #[case(1, include_str!("./resources/recurring_wholeday.ics"))]
    #[case(2, include_str!("./resources/ical_recurrence_date.ics"))]
    #[case(3, include_str!("./resources/ical_recurrence_date_2.ics"))]
    // Has no RRULE
    #[case(4, include_str!("./resources/ical_example_1.ics"))]
    fn rrule_expansion(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        for (i, res) in reader.enumerate() {
            let cal = res.unwrap();
            let recurrence = cal.expand_recurrence(None, None).unwrap();
            assert!(recurrence.get_tzids().is_empty());
            insta::assert_snapshot!(format!("{i}_ics"), recurrence.generate_string());
            insta::assert_debug_snapshot!(format!("{i}_data"), recurrence.get_inner());
        }
    }

    #[rstest::rstest]
    #[case(include_str!("./resources/ical_recurrence_date_2.ics"))]
    fn rrule_expansion_empty(#[case] input: &str) {
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            let recurrence = cal.expand_recurrence(Some(DateTime::<Utc>::MAX_UTC), None);
            assert!(recurrence.is_none());
        }
    }
}

pub mod rfc7809 {
    use caldata::{IcalObjectParser, IcalParser, generator::Emitter, parser::ParserOptions};

    #[rstest::rstest]
    #[case(0, include_str!("./resources/ical_rfc7809.ics"))]
    #[case(1, include_str!("./resources/ical_rfc7809_journal.ics"))]
    #[case(2, include_str!("./resources/ical_rfc7809_todo.ics"))]
    fn rfc7809(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        assert!(reader.expect_one().is_err());
        let reader = IcalObjectParser::from_slice(input.as_bytes())
            .with_options(ParserOptions { rfc7809: true });

        let cal = reader.expect_one().unwrap();
        insta::assert_snapshot!(cal.generate_string());

        let reader = IcalParser::from_slice(input.as_bytes());
        assert!(reader.expect_one().is_err());
        let reader =
            IcalParser::from_slice(input.as_bytes()).with_options(ParserOptions { rfc7809: true });

        let cal2 = reader.expect_one().unwrap();
        insta::assert_snapshot!("fullcal", cal2.generate_string());
    }

    #[test]
    fn test_tzdb_version() {
        assert_eq!(
            chrono_tz::IANA_TZDB_VERSION,
            vtimezones_rs::IANA_TZDB_VERSION
        );
    }
}

pub mod parser {
    use caldata::{IcalObjectParser, IcalParser, component::IcalCalendar, generator::Emitter};
    use itertools::Itertools;
    use rstest::rstest;

    use crate::utils::str_normalise_prop_order;

    #[rstest]
    #[case(0, include_str!("./resources/ical_events.ics"))]
    #[case(1, include_str!("./resources/ical_everything.ics"))]
    #[case(2, include_str!("./resources/ical_multiple.ics"))]
    #[case(3, include_str!("./resources/ical_example_1.ics"))]
    #[case(4, include_str!("./resources/ical_example_2.ics"))]
    fn ical_parse(#[case] case: usize, #[case] ical: &str) {
        set_snapshot_suffix!("{case}");
        let reader = IcalParser::from_slice(ical.as_bytes());
        let cals: Vec<IcalCalendar> = reader.map(Result::unwrap).collect();
        insta::assert_debug_snapshot!(cals);
    }

    #[rstest]
    #[case(include_str!("./resources/ical_events.ics"))]
    #[case(include_str!("./resources/ical_special_symbols.ics"))]
    #[case(include_str!("./resources/ical_todos.ics"))]
    #[case(include_str!("./resources/ical_journals.ics"))]
    #[case(include_str!("./resources/ical_freebusy.ics"))]
    #[case(include_str!("./resources/o365_meeting.ics"))]
    #[case(include_str!("./resources/sabre_test.ics"))]
    #[case(include_str!("./resources/ical_example_rrule.ics"))]
    fn ical_roundtrip(#[case] ical: &str) {
        let reader = IcalParser::from_slice(ical.as_bytes());
        let cals: Vec<IcalCalendar> = reader.map(Result::unwrap).collect();
        similar_asserts::assert_eq!(cals.iter().map(Emitter::generate_string).join(""), ical);
    }

    #[test]
    fn ical_export() {
        let input1 = include_str!("./resources/ical_events.ics");
        let input2 = include_str!("./resources/ical_example_1.ics");
        let input3 = include_str!("./resources/ical_example_rrule.ics");
        let cal1 = IcalObjectParser::from_slice(input1.as_bytes())
            .expect_one()
            .unwrap();
        let cal2 = IcalObjectParser::from_slice(input2.as_bytes())
            .expect_one()
            .unwrap();
        let cal3 = IcalObjectParser::from_slice(input3.as_bytes())
            .expect_one()
            .unwrap();
        let export = IcalCalendar::from_objects(
            "caldata-rs test".to_owned(),
            vec![cal1.to_owned(), cal2.to_owned(), cal3.to_owned()],
            vec![],
        )
        .generate_string();
        insta::assert_snapshot!(export);
        // Ensure that exported calendar is valid
        let roundtrip_cal = IcalParser::from_slice(export.as_bytes())
            .expect_one()
            .unwrap();

        let mut reference = vec![cal1, cal2, cal3];
        let mut reimported = roundtrip_cal.into_objects().unwrap();
        reference.sort_by_key(|o| o.get_uid().to_owned());
        reimported.sort_by_key(|o| o.get_uid().to_owned());
        assert_eq!(reimported.len(), reference.len());
        for (mut reference, mut reimported) in reference.into_iter().zip(reimported) {
            // PRODID gets overwritten
            reference.properties = vec![];
            reimported.properties = vec![];
            similar_asserts::assert_eq!(
                str_normalise_prop_order(&reference.generate_string()),
                str_normalise_prop_order(&reimported.generate_string())
            );
        }
    }
}

pub mod chrono_tz {
    use caldata::component::IcalTimeZone;
    use caldata::parser::ComponentParser;
    use rstest::rstest;
    const VTIMEZONE_DIFFERENT_TZID_BERLIN: &str = r#"
BEGIN:VTIMEZONE
TZID:HELLO_Europe/Berlin
LAST-MODIFIED:20250723T154628Z
X-LIC-LOCATION:Europe/Berlin
BEGIN:DAYLIGHT
TZNAME:CEST
TZOFFSETFROM:+0100
TZOFFSETTO:+0200
DTSTART:19700329T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CET
TZOFFSETFROM:+0200
TZOFFSETTO:+0100
DTSTART:19701025T030000
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
END:STANDARD
END:VTIMEZONE
    "#;

    const VTIMEZONE_BERLIN: &str = r#"
BEGIN:VTIMEZONE
TZID:Europe/Berlin
LAST-MODIFIED:20250723T154628Z
X-LIC-LOCATION:Europe/Berlin
BEGIN:DAYLIGHT
TZNAME:CEST
TZOFFSETFROM:+0100
TZOFFSETTO:+0200
DTSTART:19700329T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CET
TZOFFSETFROM:+0200
TZOFFSETTO:+0100
DTSTART:19701025T030000
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
END:STANDARD
END:VTIMEZONE
    "#;

    const VTIMEZONE_LOWERCASE: &str = r#"
BEGIN:VTIMEZONE
tzid:W. Europe Standard Time
LAST-MODIFIED:20250723T154628Z
BEGIN:DAYLIGHT
TZNAME:CEST
TZOFFSETFROM:+0100
TZOFFSETTO:+0200
DTSTART:19700329T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CET
TZOFFSETFROM:+0200
TZOFFSETTO:+0100
DTSTART:19701025T030000
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
END:STANDARD
END:VTIMEZONE
    "#;

    const VTIMEZONE_PROPRIETARY: &str = r#"
BEGIN:VTIMEZONE
TZID:W. Europe Standard Time
LAST-MODIFIED:20250723T154628Z
BEGIN:DAYLIGHT
TZNAME:CEST
TZOFFSETFROM:+0100
TZOFFSETTO:+0200
DTSTART:19700329T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CET
TZOFFSETFROM:+0200
TZOFFSETTO:+0100
DTSTART:19701025T030000
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
END:STANDARD
END:VTIMEZONE
    "#;

    #[rstest]
    #[case(VTIMEZONE_BERLIN, chrono_tz::Europe::Berlin)]
    #[case(VTIMEZONE_DIFFERENT_TZID_BERLIN, chrono_tz::Europe::Berlin)]
    #[case(VTIMEZONE_LOWERCASE, chrono_tz::Europe::Berlin)]
    #[case(VTIMEZONE_PROPRIETARY, chrono_tz::Europe::Berlin)]
    fn try_from_icaldatetime(#[case] input: &str, #[case] tz: chrono_tz::Tz) {
        let vtimezone: IcalTimeZone =
            ComponentParser::<'_, IcalTimeZone, _>::from_slice(input.as_bytes())
                .next()
                .unwrap()
                .unwrap();
        let extracted_tz: Option<chrono_tz::Tz> = (&vtimezone).into();
        assert_eq!(tz, extracted_tz.unwrap());
    }
}
