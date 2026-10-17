use rstest::rstest;

use crate::{
    ContentLineParser,
    parser::{ContentLine, Line},
};

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

#[rstest]
#[case("tEsT")]
#[case("hello")]
fn test_name_uppercase(#[case] name: &str) {
    let content_line = ContentLine::parse_line(Line {
        inner: format!("{}:value", name).into(),
        number: 0,
    })
    .unwrap();
    assert_eq!(content_line.name, content_line.name.to_uppercase())
}

#[rstest]
#[case("tEsT")]
#[case("hello")]
fn test_params_uppercase(#[case] name: &str) {
    let content_line = ContentLine::parse_line(Line {
        inner: format!("prop;{}=123:value", name).into(),
        number: 0,
    })
    .unwrap();
    let params = content_line.params.0;
    let param_name = params[0].to_owned().0;
    assert_eq!(param_name, param_name.to_uppercase())
}
