use crate::types::CalDateOrDateTime;

super::property!(
    "DTSTART",
    "DATE-TIME",
    IcalDTSTARTProperty,
    CalDateOrDateTime
);

#[cfg(test)]
mod tests {
    use super::IcalDTSTARTProperty;
    use crate::{
        generator::Emitter,
        parser::{ContentLineParams, ICalProperty},
        property::ContentLine,
        types::{CalDateOrDateTime, Tz},
    };
    use chrono::{DateTime, TimeZone};
    use rstest::rstest;
    use std::collections::HashMap;

    #[rstest]
    #[case("DTSTART:19980118T073000Z\r\n")]
    #[case("DTSTART;TZID=Europe/Berlin:19980118T073000Z\r\n")]
    #[case("DTSTART;TZID=W. Europe Standard Time:20210527T120000\r\n")]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::ContentLineParser::from_slice(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let mut timezones = HashMap::new();
        timezones.insert("Europe/Berlin".to_owned(), Some(chrono_tz::Europe::Berlin));
        timezones.insert("W. Europe Standard Time".to_owned(), None);
        let prop = IcalDTSTARTProperty::parse_prop(&content_line, Some(&timezones)).unwrap();
        let roundtrip: ContentLine = prop.into();
        similar_asserts::assert_eq!(roundtrip.generate(), input);
    }

    #[rstest]
    #[case(Tz::Olson(chrono_tz::Europe::Berlin)
            .with_ymd_and_hms(2021, 1, 1, 0, 0, 0)
            .unwrap(), "DTSTART;TZID=Europe/Berlin:20210101T000000\r\n")]
    #[case(Tz::Local
            .with_ymd_and_hms(2021, 1, 1, 0, 0, 0)
            .unwrap(), "DTSTART:20210101T000000\r\n")]
    #[case(Tz::Olson(chrono_tz::UTC)
            .with_ymd_and_hms(2021, 1, 1, 0, 0, 0)
            .unwrap(), "DTSTART:20210101T000000Z\r\n")]
    fn test_tzid_content_line(#[case] input: DateTime<Tz>, #[case] expected: &'static str) {
        let cal_datetime: CalDateOrDateTime = input.into();
        let timezone = cal_datetime.timezone();
        let mut params = vec![];
        if let Some(tzid) = timezone.tzid() {
            params.push(("TZID".to_owned(), vec![tzid.to_owned()]));
        }
        let prop: IcalDTSTARTProperty =
            IcalDTSTARTProperty(cal_datetime, ContentLineParams(params));
        let content_line: ContentLine = prop.into();
        similar_asserts::assert_eq!(content_line.generate(), expected);
    }
}
