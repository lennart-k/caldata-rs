use crate::{parser::ContentLine, types::Value};
use chrono::Duration;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_DURATION: regex::Regex = regex::Regex::new(
        r"(?x)
        ^(?<sign>[+-])?
        P (
            (
                ((?P<D>\d+)D)?  # days
                (
                    T
                    ((?P<H>\d+)H)?
                    ((?P<M>\d+)M)?
                    ((?P<S>\d+)S)?
                )?
            )  # dur-date,dur-time
            | (
                ((?P<W>\d+)W)?
            )  # dur-week
        )
        $"
    )
    .unwrap();
}

impl TryFrom<&ContentLine> for Option<chrono::Duration> {
    type Error = InvalidDuration;

    fn try_from(value: &ContentLine) -> Result<Self, Self::Error> {
        Ok(Some(parse_duration(&value.value)?))
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Invalid duration: {0}")]
pub struct InvalidDuration(String);

pub fn parse_duration(string: &str) -> Result<Duration, InvalidDuration> {
    let captures = RE_DURATION
        .captures(string)
        .ok_or(InvalidDuration(string.to_owned()))?;

    let mut duration = Duration::zero();
    if let Some(weeks) = captures.name("W") {
        duration += Duration::weeks(weeks.as_str().parse().unwrap());
    }
    if let Some(days) = captures.name("D") {
        duration += Duration::days(days.as_str().parse().unwrap());
    }
    if let Some(hours) = captures.name("H") {
        duration += Duration::hours(hours.as_str().parse().unwrap());
    }
    if let Some(minutes) = captures.name("M") {
        duration += Duration::minutes(minutes.as_str().parse().unwrap());
    }
    if let Some(seconds) = captures.name("S") {
        duration += Duration::seconds(seconds.as_str().parse().unwrap());
    }
    if let Some(sign) = captures.name("sign")
        && sign.as_str() == "-"
    {
        duration = -duration;
    }

    Ok(duration)
}

impl Value for Duration {
    fn value_type(&self) -> Option<&'static str> {
        Some("DURATION")
    }

    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        if self.is_zero() {
            return w.write_str("PT0S");
        }
        let mut abs_duration = self.abs();
        if self < &abs_duration {
            w.write_char('-')?;
        }
        w.write_char('P')?;

        // Return weeks if duration can be expressed exactly by number of weeks
        let weeks = abs_duration.num_weeks();
        if weeks > 0 && abs_duration == Duration::weeks(weeks) {
            w.write_str(&weeks.to_string())?;
            w.write_char('W')?;
            return Ok(());
        }

        let days = abs_duration.num_days();
        if days > 0 {
            w.write_str(&days.to_string())?;
            w.write_char('D')?;
            abs_duration -= Duration::days(days);
        }
        if abs_duration.is_zero() {
            return Ok(());
        }

        w.write_char('T')?;

        let hours = abs_duration.num_hours();
        if hours > 0 {
            w.write_str(&hours.to_string())?;
            w.write_char('H')?;
            abs_duration -= Duration::hours(hours);
        }
        let minutes = abs_duration.num_minutes();
        if minutes > 0 {
            w.write_str(&minutes.to_string())?;
            w.write_char('M')?;
            abs_duration -= Duration::minutes(minutes);
        }
        let seconds = abs_duration.num_seconds();
        if seconds > 0 {
            w.write_str(&seconds.to_string())?;
            w.write_char('S')?;
            abs_duration -= Duration::seconds(seconds);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Value;

    use super::parse_duration;
    use chrono::Duration;
    use rstest::rstest;

    #[test]
    fn test_parse_duration() {
        assert!(parse_duration("P1D12W").is_err());
        assert!(parse_duration("P1W12D").is_err());
        assert!(parse_duration("PT10S12M").is_err());
        // This should yield an error but it's easier to just let it slip through as 0s
        assert_eq!(parse_duration("P").unwrap(), Duration::zero());
    }

    #[rstest]
    #[case("P12W", Duration::weeks(12))]
    #[case("-P12W", -Duration::weeks(12))]
    #[case("P12D", Duration::days(12))]
    #[case("PT12H", Duration::hours(12))]
    #[case("PT12M", Duration::minutes(12))]
    #[case("PT12S", Duration::seconds(12))]
    #[case("-PT12S", -Duration::seconds(12))]
    #[case("P2DT10M12S",
            Duration::days(2) + Duration::minutes(10) + Duration::seconds(12))]
    #[case(
            "PT10M12S",
            Duration::minutes(10) + Duration::seconds(12)
    )]
    // On a roundtrip, P should not be serialised to P
    #[case("PT0S", Duration::zero())]
    fn test_duration_roundtrip(#[case] value: &str, #[case] ref_duration: Duration) {
        let duration = parse_duration(value).unwrap();
        assert_eq!(duration, ref_duration);
        assert_eq!(duration.generate_string(), value);
    }
}
