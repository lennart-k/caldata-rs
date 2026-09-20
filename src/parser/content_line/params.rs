use derive_more::From;
use itertools::Itertools;

use crate::{PARAM_DELIMITER, PARAM_VALUE_DELIMITER, generator::Emitter};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, From)]
pub struct ContentLineParams(pub(crate) Vec<(String, Vec<String>)>);

impl ContentLineParams {
    #[inline]
    pub fn get_param(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(key, _)| name == key)
            .and_then(|(_, value)| value.iter().map(String::as_ref).next())
    }

    #[inline]
    pub fn get_tzid(&self) -> Option<&str> {
        self.get_param("TZID")
    }

    #[inline]
    pub fn get_value_type(&self) -> Option<&str> {
        self.get_param("VALUE")
    }

    pub fn replace_param(&mut self, name: String, value: String) {
        if let Some(pos) = self.0.iter().position(|(n, _)| n == &name) {
            self.0[pos] = (name, vec![value]);
        } else {
            self.0.push((name, vec![value]));
        }
    }

    #[inline]
    pub fn remove(&mut self, name: &str) {
        self.0.retain(|(n, _)| n != name);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Emitter for ContentLineParams {
    fn generate(&self) -> String {
        if self.is_empty() {
            return "".to_owned();
        }

        self.0
            .iter()
            .map(|(name, values)| {
                let value: String = values
                    .iter()
                    .map(|value| protect_param(value, false))
                    .join(&PARAM_VALUE_DELIMITER.to_string());
                format!("{name}={value}")
            })
            .join(&PARAM_DELIMITER.to_string())
    }
}

fn protect_param(param: &str, mut quoted: bool) -> String {
    let mut escaped = String::with_capacity(param.len() + 2);
    quoted |= param.contains([';', ':', ',']);
    if quoted {
        escaped.push('"');
    }
    for char in param.chars() {
        match char {
            '\n' => {
                escaped.push_str("^n");
            }
            '^' => {
                escaped.push_str("^^");
            }
            '"' => {
                escaped.push_str("^'");
            }
            _ => {
                escaped.push(char);
            }
        }
    }
    if quoted {
        escaped.push('"');
    }
    escaped
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        generator::Emitter,
        parser::{ContentLine, ContentLineParams, Line},
    };

    #[rstest]
    #[case("^^", "^")]
    #[case("^'", "\"")]
    #[case("^n", "\n")]
    #[case("^a", "^a")]
    #[case("^t", "^t")]
    fn test_parse_params(#[case] input: &str, #[case] parsed: &str) {
        // Test unquoted
        assert_eq!(
            ContentLine::parse_line(Line {
                inner: format!(r#"HALLO;ASD={input}:NICE"#).into(),
                number: 1,
            })
            .unwrap()
            .params
            .get_param("ASD"),
            Some(parsed)
        );

        // Test dquoted
        assert_eq!(
            ContentLine::parse_line(Line {
                inner: format!(r#"HALLO;ASD="{input}":NICE"#).into(),
                number: 1,
            })
            .unwrap()
            .params
            .get_param("ASD"),
            Some(parsed)
        );
    }

    #[rstest]
    #[case("a\nn", "a^nn")]
    #[case("a\"n", "a^'n")]
    #[case("\t", "\t")]
    #[case(
        "42 Plantation St.\nBaytown, LA 30314\nUnited States of America",
        "\"42 Plantation St.^nBaytown, LA 30314^nUnited States of America\""
    )]
    #[case("ÄÖsÜa,ßø", "\"ÄÖsÜa,ßø\"")]
    // The param values MUST be escaped
    #[case("a\n;n", "\"a^n;n\"")]
    #[case("a\n,n", "\"a^n,n\"")]
    #[case("a\n:n", "\"a^n:n\"")]
    fn test_generate_params_unquoted(#[case] value: &str, #[case] escaped: &str) {
        assert_eq!(
            ContentLineParams(vec![("HALLO".to_owned(), vec![value.to_owned()])]).generate(),
            format!("HALLO={escaped}")
        );
    }

    #[rstest]
    #[case("DESCRIPTION;ALTREP=\"data:text/html,Hello%20World\":Hello World")]
    fn test_content_line_roundtrip(#[case] content_line: &str) {
        assert_eq!(
            ContentLine::parse_line(Line {
                inner: content_line.into(),
                number: 1,
            })
            .unwrap()
            .generate(),
            format!("{content_line}\r\n")
        )
    }
}
