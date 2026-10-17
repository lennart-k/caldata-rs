use crate::ContentLineParser;

#[test]
fn ical() {
    let input = include_str!("../../../tests/resources/ical_multiple.ics");
    let reader = ContentLineParser::from_slice(input.as_bytes());
    let lines = reader.collect::<Result<Vec<_>, _>>().unwrap();
    insta::assert_debug_snapshot!(lines);
}

#[test]
fn vcard() {
    let input = include_str!("../../../tests/resources/vcard_input.vcf");
    let reader = ContentLineParser::from_slice(input.as_bytes());
    let lines = reader.collect::<Result<Vec<_>, _>>().unwrap();
    insta::assert_debug_snapshot!(lines);
}

#[test]
fn errors() {
    let input = include_str!("../../../tests/resources/property_error.vcf");
    let reader = ContentLineParser::from_slice(input.as_bytes());
    for res in reader {
        assert!(res.is_err());
    }
}
