use crate::{
    parser::ContentLineError,
    types::{CalDateTimeError, InvalidDuration},
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParserError {
    #[error("empty input")]
    EmptyInput,
    #[error("too many components in input, expected one")]
    TooManyComponents,
    #[error("invalid component: {0}")]
    InvalidComponent(String),
    #[error("incomplete object")]
    NotComplete,
    #[error("missing header")]
    MissingHeader,
    #[error("content line error: {0}")]
    ContentLineError(#[from] ContentLineError),
    #[error("missing property: {0}")]
    MissingProperty(&'static str),
    #[error("missing property: UID")]
    MissingUID,
    #[error("property conflict: {0}")]
    PropertyConflict(&'static str),
    #[error(transparent)]
    InvalidDuration(#[from] InvalidDuration),
    #[error("invalid property value: {0}")]
    InvalidPropertyValue(String),
    #[error("invalid property value type for: {0}")]
    InvalidPropertyType(String),
    #[error(transparent)]
    RRule(#[from] rrule::RRuleError),
    #[error(transparent)]
    DateTime(#[from] CalDateTimeError),
    #[error("Invalid CALSCALE: Only GREGORIAN supported")]
    InvalidCalscale,
    #[error("Invalid VERSION: MUST be 1.0 or 2.0")]
    InvalidVersion,
    #[error("Multiple main events are not allowed in a calendar object")]
    MultipleMainObjects,
    #[error("Differing UIDs inside a calendar object")]
    DifferingUIDs,
    #[error("Override without RECURRENCE-ID")]
    MissingRecurId,
    #[error("DTSTART and RECURRENCE-ID must have the same value type and timezone")]
    DtstartNotMatchingRecurId,
}
