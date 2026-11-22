use crate::types::CalDateOrDateTime;
super::property!("DUE", "DATE-TIME", IcalDUEProperty, CalDateOrDateTime);

#[cfg(test)]
mod tests {
    use super::IcalDUEProperty;
    use crate::{generator::Emitter, parser::ICalProperty, property::ContentLine};
    use rstest::rstest;

    #[rstest]
    #[case("DUE:19960402T010000Z\r\n")]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::ContentLineParser::from_slice(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let prop = IcalDUEProperty::parse_prop(content_line, None).unwrap();
        similar_asserts::assert_eq!(prop.generate_string(), input);
        let content_line: ContentLine = prop.into();
        similar_asserts::assert_eq!(content_line.generate_string(), input);
    }
}
