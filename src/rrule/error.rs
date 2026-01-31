/*
 * SPDX-FileCopyrightText: 2021 Fredrik Meringdal, Ralph Bisschops <https://github.com/fmeringdal/rust-rrule>
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 *
 * This code is taken from github.com/fmeringdal/rust-rrule with slight modifications.
 */
#![allow(clippy::module_name_repetitions)]

use crate::rrule::Frequency;
pub use crate::rrule::parser::ParseError;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
/// The error type for the rrule crate.
pub enum RRuleError {
    /// Parsing error
    #[error("RRule parsing error: {0}")]
    ParserError(#[from] ParseError),
    /// Validation error
    #[error("RRule validation error: {0}")]
    ValidationError(#[from] ValidationError),
    /// Iterator error
    #[error("RRule iterator error: {0}")]
    IterError(String),
}

impl RRuleError {
    /// Create a new iterator error with the given message.
    pub fn new_iter_err<S: AsRef<str>>(msg: S) -> Self {
        Self::IterError(msg.as_ref().to_owned())
    }
}

#[allow(missing_docs)]
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("BYSETPOS should only be used in conjunction with another BYxxx rule part.")]
    BySetPosWithoutByRule,
    #[error("`{field}` can't be `{value}`, must be larger or smaller then `{value}`.")]
    InvalidFieldValue { field: String, value: String },
    #[error(
        "`{field}` is `{value}`, but is not allowed outside the range: `{start_idx}..={end_idx}`."
    )]
    InvalidFieldValueRange {
        field: String,
        value: String,
        start_idx: String,
        end_idx: String,
    },
    #[error(
        "`{field}` is `{value}`, but with the current frequency ({freq}) is not allowed \
            outside the range: `{start_idx}..={end_idx}`."
    )]
    InvalidFieldValueRangeWithFreq {
        field: String,
        value: String,
        freq: Frequency,
        start_idx: String,
        end_idx: String,
    },
    #[error("`{by_rule}` can not be used with the current frequency ({freq}).")]
    InvalidByRuleAndFrequency { by_rule: String, freq: Frequency },
    #[error(
        "`UNTIL` is `{until}`, but `DTSTART` (`{dt_start}`) is later. That should not be happening."
    )]
    UntilBeforeStart { until: String, dt_start: String },
    #[error(
        "`INTERVAL` is `{0}`, is higher than expected, make sure this is correct. \
            See 'validator limits' in docs for more info."
    )]
    TooBigInterval(u16),
    #[error(
        "`DTSTART` year is `{0}`, is higher/lower than expected, make sure this is correct. \
            See 'validator limits' in docs for more info."
    )]
    StartYearOutOfRange(i32),
    #[error(
        "Unable to generate a timeset for the RRULE. Please specify a BYHOUR, BYMINUTE or BYSECOND"
    )]
    UnableToGenerateTimeset,
    #[error("`BYEASTER` can only be used when `BYHOUR`, `BYMINUTE` and `BYSECOND` are set.")]
    InvalidByRuleWithByEaster,
    #[error(
        "The value of `DTSTART` was specified in {dt_start_tz} timezone, but `UNTIL` was specified in timezone {until_tz}. Allowed timezones for `UNTIL` with the given start date timezone are: `{expected:?}`"
    )]
    DtStartUntilMismatchTimezone {
        dt_start_tz: String,
        until_tz: String,
        expected: Vec<String>,
    },
}
