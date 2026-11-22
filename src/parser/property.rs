pub trait ParseProp: Sized {
    fn parse_prop(
        prop: &ContentLine,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        default_type: &str,
    ) -> Result<Self, ParserError>;
}

impl ParseProp for String {
    fn parse_prop(
        prop: &ContentLine,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Ok(prop.value.to_owned())
    }
}

impl ParseProp for DateOrDateTimeOrPeriod {
    fn parse_prop(
        prop: &ContentLine,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        default_type: &str,
    ) -> Result<Self, ParserError> {
        Self::parse_prop(prop, timezones, default_type)
    }
}

impl ParseProp for CalDateOrDateTime {
    fn parse_prop(
        prop: &ContentLine,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        default_type: &str,
    ) -> Result<Self, ParserError> {
        Self::parse_prop(prop, timezones, default_type)
    }
}

impl ParseProp for CalDateTime {
    fn parse_prop(
        prop: &ContentLine,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Self::parse_prop(prop, timezones)
    }
}

impl ParseProp for chrono::Duration {
    fn parse_prop(
        prop: &ContentLine,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Ok(parse_duration(&prop.value)?)
    }
}

impl ParseProp for crate::rrule::RRule<crate::rrule::Unvalidated> {
    fn parse_prop(
        prop: &ContentLine,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Ok(crate::rrule::RRule::from_str(&prop.value)?)
    }
}

impl<T: ParseProp> ParseProp for Vec<T> {
    fn parse_prop(
        prop: &ContentLine,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        default_type: &str,
    ) -> Result<Self, ParserError> {
        let mut out = vec![];
        for value in prop.value.trim_end_matches(',').split(',') {
            let content_line = ContentLine {
                name: prop.name.to_owned(),
                params: prop.params.to_owned(),
                value: value.to_owned(),
            };
            out.push(T::parse_prop(&content_line, timezones, default_type)?);
        }
        Ok(out)
    }
}

pub trait ICalProperty: Sized + Emitter + GetTzid {
    const NAME: &'static str;
    const DEFAULT_TYPE: &'static str;

    fn parse_prop(
        prop: ContentLine,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self, ParserError>;

    fn utc_or_local(self) -> Self;
}

pub trait GetTzid {
    fn get_tzid(&self) -> Option<&str>;
}

impl GetTzid for ContentLine {
    fn get_tzid(&self) -> Option<&str> {
        self.params.get_tzid()
    }
}

macro_rules! property {
    ($name:literal, $default_type:literal, $prop:ty) => {
        impl $crate::parser::ICalProperty for $prop {
            const NAME: &'static str = $name;
            const DEFAULT_TYPE: &'static str = $default_type;

            #[inline]
            fn parse_prop(
                prop: crate::parser::ContentLine,
                timezones: Option<&std::collections::HashMap<String, Option<chrono_tz::Tz>>>,
            ) -> Result<Self, crate::parser::ParserError> {
                Ok(Self(
                    crate::parser::ParseProp::parse_prop(&prop, timezones, $default_type)?,
                    prop.params,
                ))
            }

            #[inline]
            fn utc_or_local(self) -> Self {
                let Self(dt, mut params) = self;
                params.remove("TZID");
                Self(crate::types::Value::utc_or_local(dt), params)
            }
        }

        impl $crate::parser::GetTzid for $prop {
            #[inline]
            fn get_tzid(&self) -> Option<&str> {
                self.1.get_tzid()
            }
        }

        impl $crate::generator::Emitter for $prop {
            fn generate<W: ::std::fmt::Write>(&self, i: &mut W) -> ::std::fmt::Result {
                use ::std::fmt::Write;
                let mut w = $crate::generator::LineWriter::new(i);
                w.write_str($name)?;
                let Self(inner, params) = self;
                let type_param = if let Some(value_type) = crate::types::Value::value_type(inner)
                    && value_type != $default_type
                    && params.get_value_type().is_none()
                {
                    Some(value_type)
                } else {
                    None
                };

                if let Some(type_param) = type_param {
                    w.write_char($crate::PARAM_DELIMITER)?;
                    w.write_str("VALUE=")?;
                    w.write_str(type_param)?;
                }
                if !params.is_empty() {
                    if type_param.is_some() {
                        w.write_char($crate::PARAM_VALUE_DELIMITER)?;
                    } else {
                        w.write_char($crate::PARAM_DELIMITER)?;
                    }
                    params.generate(&mut w)?;
                }
                w.write_char($crate::VALUE_DELIMITER)?;
                w.write_str(&$crate::types::Value::value(inner))?;
                w.write_str("\r\n")?;
                Ok(())
            }
        }
    };

    ($name:literal, $default_type:literal, $prop:ident, $inner:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, derive_more::From)]
        pub struct $prop(pub $inner, pub crate::parser::ContentLineParams);
        crate::parser::property!($name, $default_type, $prop);

        impl From<$prop> for crate::parser::ContentLine {
            fn from(prop: $prop) -> Self {
                let $prop(inner, mut params) = prop;
                let value_type = crate::types::Value::value_type(&inner).unwrap_or($default_type);
                if value_type != $default_type {
                    params.replace_param("VALUE".to_owned(), value_type.to_owned());
                }
                crate::parser::ContentLine {
                    name: $name.to_owned(),
                    params,
                    value: crate::types::Value::value(&inner),
                }
            }
        }
    };
}
use std::{collections::HashMap, str::FromStr};

pub(crate) use property;

use crate::{
    ParserError,
    generator::Emitter,
    parser::ContentLine,
    types::{CalDateOrDateTime, CalDateTime, DateOrDateTimeOrPeriod, parse_duration},
};
