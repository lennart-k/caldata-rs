use std::borrow::Cow;
use std::iter::Iterator;

use super::{BytesLines, Line, LineError, LineReader};
use crate::{PARAM_DELIMITER, PARAM_NAME_DELIMITER, PARAM_VALUE_DELIMITER, VALUE_DELIMITER};

mod generator;
mod params;
pub use params::*;

/// Error arising when trying to parse a content line
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ContentLineError {
    #[error("Line {0}: Missing property name.")]
    MissingName(usize),
    #[error("Line {0}: Missing a closing quote.")]
    MissingClosingQuote(usize),
    #[error("Line {0}: Missing a \"{1}\" delimiter.")]
    MissingDelimiter(usize, char),
    #[error("Line {0}: Missing content after \"{1}\".")]
    MissingContentAfter(usize, char),
    #[error("Line {0}: Missing a parameter key.")]
    MissingParamKey(usize),
    #[error("Line {0}: Missing value.")]
    MissingValue(usize),
    #[error(transparent)]
    LineError(#[from] LineError),
}

/// A VCARD/ICAL property.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct ContentLine {
    /// Property name.
    pub name: String,
    /// Property list of parameters.
    pub params: ContentLineParams,
    /// Property value.
    pub value: String,
}

impl ContentLine {
    pub fn parse_line(line: Line) -> Result<Self, ContentLineError> {
        let mut to_parse = line.as_str();

        // Find end of parameter name
        let Some(param_end_pos) = to_parse.find([PARAM_DELIMITER, VALUE_DELIMITER]) else {
            return Err(ContentLineError::MissingName(line.number()));
        };
        let (prop_name, remainder) = to_parse.split_at(param_end_pos);
        if prop_name.is_empty() {
            return Err(ContentLineError::MissingName(line.number()));
        }
        to_parse = remainder;

        // remainder either starts with ; or :
        // Fetch all parameters
        let mut params = vec![];
        while to_parse.starts_with(PARAM_DELIMITER) {
            to_parse = &to_parse[1..];

            // Split the param key and the rest of the line
            let Some((key, remainder)) = to_parse.split_once(PARAM_NAME_DELIMITER) else {
                return Err(ContentLineError::MissingDelimiter(
                    line.number(),
                    PARAM_NAME_DELIMITER,
                ));
            };
            if key.is_empty() {
                return Err(ContentLineError::MissingParamKey(line.number()));
            }
            to_parse = remainder;

            // In almost all cases we'll have one parameter value
            let mut values = Vec::with_capacity(1);

            // Loop over comma-separated parameter values
            loop {
                if to_parse.starts_with('"') {
                    // This is a dquoted value. (NAME:Foo="Bar":value)
                    // Skip first dquote
                    to_parse = &to_parse[1..];
                    let Some((content, remainder)) = to_parse.split_once('"') else {
                        return Err(ContentLineError::MissingClosingQuote(line.number()));
                    };
                    values.push(unescape_param(content));
                    to_parse = remainder;
                } else {
                    // This is a 'raw' value. (NAME;Foo=Bar:value)
                    // Try to find the next param separator.
                    let Some(delim_pos) =
                        to_parse.find([PARAM_DELIMITER, VALUE_DELIMITER, PARAM_VALUE_DELIMITER])
                    else {
                        return Err(ContentLineError::MissingContentAfter(
                            line.number(),
                            PARAM_NAME_DELIMITER,
                        ));
                    };
                    let (content, remainder) = to_parse.split_at(delim_pos);

                    values.push(unescape_param(content));
                    to_parse = remainder;
                }

                if !to_parse.starts_with(PARAM_VALUE_DELIMITER) {
                    break;
                }
                to_parse = &to_parse[1..];
            }

            params.push((key.to_uppercase(), values));
        }

        // Parse value
        if !to_parse.starts_with(VALUE_DELIMITER) {
            return Err(ContentLineError::MissingValue(line.number()));
        }
        to_parse = &to_parse[1..];
        Ok(ContentLine {
            name: prop_name.to_uppercase(),
            params: params.into(),
            value: to_parse.to_owned(),
        })
    }
}

fn unescape_param(s: &str) -> String {
    let mut result = String::with_capacity(s.len()); // Pre-allocate
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '^' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('^') => result.push('^'),
                Some('\'') => result.push('"'),
                Some(other) => {
                    result.push('^');
                    result.push(other);
                }
                None => result.push('^'),
            }
        } else {
            result.push(ch);
        }
    }

    result
}

pub struct ContentLineParser<'a, T: Iterator<Item = Cow<'a, [u8]>>>(LineReader<'a, T>);

impl<'a> ContentLineParser<'a, BytesLines<'a>> {
    pub fn from_slice(slice: &'a [u8]) -> Self {
        ContentLineParser(LineReader::from_slice(slice))
    }
}

impl<'a, T: Iterator<Item = Cow<'a, [u8]>>> ContentLineParser<'a, T> {
    pub fn new(line_reader: LineReader<'a, T>) -> Self {
        ContentLineParser(line_reader)
    }
}

impl<'a, T: Iterator<Item = Cow<'a, [u8]>>> Iterator for ContentLineParser<'a, T> {
    type Item = Result<ContentLine, ContentLineError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(Ok(line)) => Some(ContentLine::parse_line(line)),
            Some(Err(err)) => Some(Err(err.into())),
            None => None,
        }
    }
}
