use crate::{VcardParser, generator::Emitter};

#[test]
fn vcard() {
    let input = include_str!("../../../tests/resources/vcard_input.vcf");
    let reader = VcardParser::from_slice(input.as_bytes());
    let card = reader.expect_one().unwrap();
    assert_eq!(card.get_uid(), Some("jdoelaskdjlaksjd"))
}

#[test]
fn vcard_lowercase() {
    let input = include_str!("../../../tests/resources/vcard_lowercase.vcf");
    let reader = VcardParser::from_slice(input.as_bytes());
    for res in reader {
        let card = res.unwrap();
        insta::assert_debug_snapshot!(card);
        similar_asserts::assert_eq!(card.generate().to_lowercase(), input.to_lowercase());
    }
}

#[test]
fn vcard_invalid() {
    let input = include_str!("../../../tests/resources/vcard_invalid.vcf");
    let reader = VcardParser::from_slice(input.as_bytes());
    for res in reader {
        assert!(res.is_err());
    }
}
