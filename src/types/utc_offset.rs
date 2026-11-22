use std::collections::HashMap;

use crate::{
    ParserError,
    generator::Emitter,
    parser::{ContentLine, ParseProp},
    types::Value,
};
use chrono::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcOffset(pub(crate) Duration);

impl Value for UtcOffset {
    fn value_type(&self) -> Option<&'static str> {
        Some("UTC-OFFSET")
    }

    fn value(&self) -> String {
        let sign = if self.0 >= Duration::default() {
            "+"
        } else {
            "-"
        };
        let total_secs = self.0.num_seconds().unsigned_abs();
        let (hours, mins, secs) = (total_secs / 3600, (total_secs / 60) % 60, total_secs % 60);
        if secs == 0 {
            format!("{sign}{hours:02}{mins:02}",)
        } else {
            format!("{sign}{hours:02}{mins:02}{secs:02}",)
        }
    }
}

fn parse_offset_value(input: &str) -> Result<Duration, ParserError> {
    let (sign, rest) = input.split_at(1);
    let sign = match sign {
        "+" => 1,
        "-" => -1,
        _ => return Err(ParserError::InvalidPropertyValue(input.to_string())),
    };

    let (hours, rest) = rest.split_at(2);
    let hours: u64 = hours
        .parse()
        .map_err(|_| ParserError::InvalidPropertyValue(input.to_string()))?;
    let (mins, rest) = rest.split_at(2);
    let mins: u64 = mins
        .parse::<u64>()
        .map_err(|_| ParserError::InvalidPropertyValue(input.to_string()))?
        + 60 * hours;
    let secs = if !rest.is_empty() {
        rest.parse::<u64>()
            .map_err(|_| ParserError::InvalidPropertyValue(input.to_string()))?
    } else {
        0
    } + mins * 60;

    Duration::new(secs as i64 * sign, 0)
        .ok_or_else(|| ParserError::InvalidPropertyValue(input.to_string()))
}

impl ParseProp for UtcOffset {
    fn parse_prop(
        prop: &ContentLine,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        default_type: &str,
    ) -> Result<Self, ParserError> {
        let value_type = prop.params.get_value_type().unwrap_or(default_type);
        match value_type {
            "UTC-OFFSET" => Ok(Self(parse_offset_value(&prop.value)?)),
            _ => Err(ParserError::InvalidPropertyType(prop.generate_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::UtcOffset;
    use crate::{
        ParserError,
        parser::{ContentLine, ParseProp},
        types::Value,
    };
    use chrono::Duration;

    #[rstest::rstest]
    #[case(Duration::new(12, 0).unwrap(), "+000012")]
    #[case(Duration::new(-12, 0).unwrap(), "-000012")]
    #[case(Duration::new(3661, 0).unwrap(), "+010101")]
    #[case(Duration::hours(10), "+1000")]
    #[case(Duration::hours(-10), "-1000")]
    #[case(Duration::minutes(10), "+0010")]
    #[case(Duration::minutes(-10), "-0010")]
    fn test_format_value(#[case] duration: Duration, #[case] out: &str) {
        similar_asserts::assert_eq!(out, &UtcOffset(duration).value());
    }

    #[test]
    fn test_parse_prop_invalid_type() {
        assert_matches!(
            UtcOffset::parse_prop(
                &ContentLine {
                    ..Default::default()
                },
                None,
                "TEXT"
            ),
            Err(ParserError::InvalidPropertyType(_))
        );
    }

    #[rstest::rstest]
    #[case(Duration::new(12, 0).unwrap(), "+000012")]
    #[case(Duration::new(-12, 0).unwrap(), "-000012")]
    #[case(Duration::new(3661, 0).unwrap(), "+010101")]
    #[case(Duration::hours(10), "+1000")]
    #[case(Duration::hours(10), "+100000")]
    #[case(Duration::hours(-10), "-1000")]
    #[case(Duration::hours(-10), "-100000")]
    #[case(Duration::minutes(10), "+0010")]
    #[case(Duration::minutes(-10), "-0010")]
    fn test_parse_prop_valid(#[case] duration: Duration, #[case] input: &str) {
        assert_eq!(
            UtcOffset::parse_prop(
                &ContentLine {
                    value: input.to_string(),
                    ..Default::default()
                },
                None,
                "UTC-OFFSET"
            )
            .unwrap()
            .0,
            duration
        );
    }
}
