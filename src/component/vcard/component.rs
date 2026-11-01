use crate::component::{Component, ComponentMut};
use crate::parser::{ContentLine, ICalProperty, ParserError, ParserOptions};
use crate::property::{
    VcardANNIVERSARYProperty, VcardBDAYProperty, VcardFNProperty, VcardNProperty, VcardUIDProperty,
};
use crate::rrule::RRuleSet;
use std::collections::HashMap;

macro_rules! component {
    (
        $name:ident,
        $comp_names:expr,
        required_once: { $($req_once_field:ident: $req_once_type:ty),* $(,)? },
        required_many: { $($req_many_field:ident: $req_many_type:ty),* $(,)? },
        optional_many: { $($opt_many_field:ident: $opt_many_type:ty),* $(,)? },
        optional_once: { $($opt_once_field:ident: $opt_once_type:ty),* $(,)? },
        components: { $($comp_field:ident: $comp_type:ty),* $(,)? },
        extra_fields: { $($extra_field:ident: $extra_field_type:ty = |$(&$extra_field_uses:ident),*| $extra_field_build:expr),* $(,)? },
    ) => {
        ::paste::paste! {
            #[derive(Debug, Clone)]
            pub struct $name {
                // Required exactly once
                $(pub $req_once_field: $req_once_type,)*
                // Required at least once
                $(pub $req_many_field: Vec<$req_many_type>,)*
                // Optional 0 or more times
                $(pub $opt_many_field: Vec<$opt_many_type>,)*
                // Optional at most once
                $(pub $opt_once_field: Option<$opt_once_type>,)*
                other_properties: Vec<ContentLine>,
                $($comp_field: Vec<$comp_type>,)*
                $($extra_field: $extra_field_type,)*
            }

            #[derive(Debug, Clone, Default)]
            pub struct [< $name Builder >] {
                content_lines: Vec<ContentLine>,
                $($comp_field: Vec<<$comp_type as Component>::Builder>),*
            }

            impl Component for [< $name Builder >] {
                const NAMES: &[&str] = $comp_names;
                type Builder = Self;
            }

            impl ComponentMut for [< $name Builder >] {
                type Verified = $name;

                fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
                    self.content_lines.push(content_line);
                    Ok(())
                }

                #[inline]
                fn add_sub_component<'a, I: Iterator<Item = ::std::borrow::Cow<'a, [u8]>>>(
                    &mut self,
                    name: &str,
                    #[allow(unused_variables)]
                    line_parser: &mut crate::ContentLineParser<'a, I>,
                    #[allow(unused_variables)]
                    options: &ParserOptions,
                ) -> Result<(), ParserError> {
                    match name {
                        $(
                            _ if $comp_type::NAMES.contains(&name) => {
                                let mut comp = $comp_type::Builder::new();
                                comp.parse(line_parser, options)?;
                                self.$comp_field.push(comp);
                                Ok(())
                            }
                        )*
                        _ => return Err(ParserError::InvalidComponent(name.to_owned())),
                    }
                }

                fn remove_property(&mut self, name: &str) {
                    self.content_lines
                        .retain(|content_line| content_line.name != name);
                }

                fn build(
                    self,
                    #[allow(unused_variables)]
                    options: &ParserOptions,
                    timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
                ) -> Result<Self::Verified, ParserError> {
                    $(let mut [< req_once_ $req_once_field >] = None;)*
                    $(let mut [< req_many_ $req_many_field >] = Vec::new();)*
                    $(let mut [< opt_once_ $opt_once_field >] = None;)*
                    $(let mut [< opt_many_ $opt_many_field >] = Vec::new();)*
                    let mut other_properties = Vec::new();

                    for content_line in self.content_lines {
                        let property_name = content_line.name.as_str();
                        match property_name {
                            $(
                                <$req_once_type as crate::parser::ICalProperty>::NAME
                                    => [< req_once_ $req_once_field >] = Some($crate::parser::ICalProperty::parse_prop(&content_line, timezones)?),
                            )*
                            $(
                                <$req_many_type as crate::parser::ICalProperty>::NAME
                                    => [< req_many_ $req_many_field >].push($crate::parser::ICalProperty::parse_prop(&content_line, timezones)?),
                            )*
                            $(
                                <$opt_once_type as crate::parser::ICalProperty>::NAME
                                    => [< opt_once_ $opt_once_field >] = Some($crate::parser::ICalProperty::parse_prop(&content_line, timezones)?),)*
                            $(
                                <$opt_many_type as crate::parser::ICalProperty>::NAME
                                    => [< opt_many_ $opt_many_field >].push($crate::parser::ICalProperty::parse_prop(&content_line, timezones)?),
                            )*
                            _ => other_properties.push(content_line)
                        }
                    }

                    $(let [< prop_ $req_once_field >] = [< req_once_ $req_once_field >].ok_or(ParserError::MissingProperty(<$req_once_type as crate::parser::ICalProperty>::NAME))?;)*
                    $(let [< prop_ $req_many_field >] = [< req_many_ $req_many_field >];)*
                    $(let [< prop_ $opt_once_field >] = [< opt_once_ $opt_once_field >];)*
                    $(let [< prop_ $opt_many_field >] = [< opt_many_ $opt_many_field >];)*

                    Ok(Self::Verified {
                        $($extra_field: (|$($extra_field_uses,)*| { $extra_field_build })($(&[< prop_ $extra_field_uses >],)*),)*
                        $($req_once_field: [< prop_ $req_once_field >],)*
                        $($req_many_field: [< prop_ $req_many_field >],)*
                        $($opt_once_field: [< prop_ $opt_once_field >],)*
                        $($opt_many_field: [< prop_ $opt_many_field >],)*
                        $(
                            $comp_field: self
                                .$comp_field
                                .into_iter()
                                .map(|comp| comp.build(options, timezones))
                                .collect::<Result<Vec<_>, _>>()?,
                        )*
                        other_properties,
                    })
                }
            }

        }
    };
}

#[derive(Debug, Clone, Component)]
pub struct VcardTestComponent {
    #[caldata(required)]
    full_name: Vec<VcardFNProperty>,
    uid: Vec<VcardUIDProperty>,
    n: Vec<VcardNProperty>,
    bday: Vec<VcardBDAYProperty>,
    anniversary: Vec<VcardANNIVERSARYProperty>,
    #[caldata(derived)]
    rruleset: RRuleSet,
}

use caldata_derive::Component;
pub(crate) use component;

// component!(
//     VcardContact,
//     &["VCARD"],
//     required_once: {
//     },
//     required_many: {
//         full_name: VcardFNProperty,
//     },
//     optional_many: {
//
//     },
//     optional_once: {
//         uid: VcardUIDProperty,
//         n: VcardNProperty,
//         bday: VcardBDAYProperty,
//         anniversary: VcardANNIVERSARYProperty,
//     },
//     components: {},
//     extra_fields: {},
// );
//
// impl VcardContact {
//     pub fn get_uid(&self) -> Option<&str> {
//         self.uid.as_ref().map(|uid| uid.0.as_str())
//     }
// }
//
// impl Component for VcardContact {
//     const NAMES: &[&str] = &["VCARD"];
//     type Builder = VcardContactBuilder;
// }
