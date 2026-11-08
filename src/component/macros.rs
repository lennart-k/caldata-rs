macro_rules! component {
    (
        $name:ident,
        $comp_name:literal,
        required_once: { $($req_once_field:ident: $req_once_type:ty),* $(,)? },
        required_many: { $($req_many_field:ident: $req_many_type:ty),* $(,)? },
        optional_many: { $($opt_many_field:ident: $opt_many_type:ty),* $(,)? },
        optional_once: { $($opt_once_field:ident: $opt_once_type:ty),* $(,)? },
        components: { $($comp_field:ident: $comp_type:ty),* $(,)? },
        extra_fields: { $($extra_field:ident: $extra_field_type:ty),* $(,)? },
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
                pub other_properties: Vec<ContentLine>,
                $($comp_field: Vec<$comp_type>,)*
                $($extra_field: $extra_field_type,)*
            }

            #[derive(Debug)]
            pub struct [< $name Fields >] {
                $(pub $req_once_field: $req_once_type,)*
                $(pub $req_many_field: Vec<$req_many_type>,)*
                $(pub $opt_many_field: Vec<$opt_many_type>,)*
                $(pub $opt_once_field: Option<$opt_once_type>,)*
                pub other_properties: Vec<ContentLine>,
                $(pub $comp_field: Vec<$comp_type>,)*
            }

            impl [< $name Fields >] {
                pub fn get_tzids(&self) -> impl Iterator<Item = &str> {
                    ::std::iter::empty()
                        $(.chain($crate::parser::GetTzid::get_tzid(&self.$req_once_field)))*
                        $(.chain(self.$req_many_field.iter().flat_map($crate::parser::GetTzid::get_tzid)))*
                        $(.chain(self.$opt_once_field.iter().flat_map($crate::parser::GetTzid::get_tzid)))*
                        $(.chain(self.$opt_many_field.iter().flat_map($crate::parser::GetTzid::get_tzid)))*
                        $(.chain(self.$comp_field.iter().flat_map($crate::component::Component::get_tzids)))*
                }
            }

            #[derive(Debug, Clone, Default)]
            pub struct [< $name Builder >] {
                content_lines: Vec<ContentLine>,
                $($comp_field: Vec<<$comp_type as Component>::Builder>),*
            }

            impl $crate::component::Component for [< $name Builder >] {
                const NAME: &str = $comp_name;
                type Builder = Self;

                fn get_tzids(&self) -> impl Iterator<Item = &str> {
                    self.content_lines.iter().flat_map($crate::parser::GetTzid::get_tzid)
                        $(.chain(self.$comp_field.iter().flat_map($crate::component::Component::get_tzids)))*
                }
            }

            impl $crate::component::Component for $name {
                const NAME: &str = $comp_name;
                type Builder = [< $name Builder >];

                fn get_tzids(&self) -> impl Iterator<Item = &str> {
                    ::std::iter::empty()
                        $(.chain($crate::parser::GetTzid::get_tzid(&self.$req_once_field)))*
                        $(.chain(self.$req_many_field.iter().flat_map($crate::parser::GetTzid::get_tzid)))*
                        $(.chain(self.$opt_once_field.iter().flat_map($crate::parser::GetTzid::get_tzid)))*
                        $(.chain(self.$opt_many_field.iter().flat_map($crate::parser::GetTzid::get_tzid)))*
                        $(.chain(self.$comp_field.iter().flat_map($crate::component::Component::get_tzids)))*
                }
            }

            impl $crate::component::ComponentMut for [< $name Builder >] {
                type Verified = $name;

                fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), $crate::ParserError> {
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
                    options: &$crate::parser::ParserOptions,
                ) -> Result<(), $crate::ParserError> {
                    match name {
                        $(
                            $comp_type::NAME => {
                                let mut comp = <$comp_type as Component>::Builder::default();
                                comp.parse(line_parser, options)?;
                                self.$comp_field.push(comp);
                                Ok(())
                            }
                        )*
                        _ => return Err($crate::ParserError::InvalidComponent(name.to_owned())),
                    }
                }

                fn build(
                    self,
                    #[allow(unused_variables)]
                    options: &$crate::parser::ParserOptions,
                    timezones: Option<&::std::collections::HashMap<String, Option<::chrono_tz::Tz>>>,
                ) -> Result<Self::Verified, $crate::ParserError> {
                    $(let mut [< req_once_ $req_once_field >] = None;)*
                    $(let mut [< req_many_ $req_many_field >] = Vec::new();)*
                    $(let mut [< opt_once_ $opt_once_field >] = None;)*
                    $(let mut [< opt_many_ $opt_many_field >] = Vec::new();)*
                    let mut other_properties = Vec::new();

                    for content_line in self.content_lines {
                        let property_name = content_line.name.as_str();
                        match property_name {
                            $(
                                <$req_once_type as $crate::parser::ICalProperty>::NAME
                                    => [< req_once_ $req_once_field >] = Some($crate::parser::ICalProperty::parse_prop(content_line, timezones)?),
                            )*
                            $(
                                <$req_many_type as $crate::parser::ICalProperty>::NAME
                                    => [< req_many_ $req_many_field >].push($crate::parser::ICalProperty::parse_prop(content_line, timezones)?),
                            )*
                            $(
                                <$opt_once_type as $crate::parser::ICalProperty>::NAME
                                    => [< opt_once_ $opt_once_field >] = Some($crate::parser::ICalProperty::parse_prop(content_line, timezones)?),)*
                            $(
                                <$opt_many_type as $crate::parser::ICalProperty>::NAME
                                    => [< opt_many_ $opt_many_field >].push($crate::parser::ICalProperty::parse_prop(content_line, timezones)?),
                            )*
                            _ => other_properties.push(content_line)
                        }
                    }

                    $(let [< prop_ $req_once_field >] = [< req_once_ $req_once_field >].ok_or($crate::ParserError::MissingProperty(<$req_once_type as $crate::parser::ICalProperty>::NAME))?;)*
                    $(let [< prop_ $req_many_field >] = [< req_many_ $req_many_field >];)*
                    $(let [< prop_ $opt_once_field >] = [< opt_once_ $opt_once_field >];)*
                    $(let [< prop_ $opt_many_field >] = [< opt_many_ $opt_many_field >];)*

                    let fields = [< $name Fields >] {
                        $($req_once_field: [< prop_ $req_once_field >],)*
                        $($req_many_field: [< prop_ $req_many_field >],)*
                        $($opt_once_field: [< prop_ $opt_once_field >],)*
                        $($opt_many_field: [< prop_ $opt_many_field >],)*
                        other_properties,
                        $(
                            $comp_field: self
                                .$comp_field
                                .into_iter()
                                .map(|comp| comp.build(options, timezones))
                                .collect::<Result<Vec<_>, _>>()?,
                        )*
                    };

                    Self::build_from_fields(fields, options)
                }
            }

            impl $crate::generator::Emitter for $name {
                fn generate<W: ::std::fmt::Write>(&self, w: &mut W) -> ::std::fmt::Result {
                    w.write_str("BEGIN:")?;
                    w.write_str(<Self as $crate::component::Component>::NAME)?;
                    w.write_str("\r\n")?;
                    $(self.$req_once_field.generate(w)?;)*
                    $(self.$opt_once_field.generate(w)?;)*
                    $(self.$req_many_field.generate(w)?;)*
                    $(self.$opt_many_field.generate(w)?;)*
                    for property in &self.other_properties {
                        property.generate(w)?;
                    }
                    $(self.$comp_field.generate(w)?;)*
                    w.write_str("END:")?;
                    w.write_str(<Self as $crate::component::Component>::NAME)?;
                    w.write_str("\r\n")?;
                    Ok(())
                }
            }
        }
    };
}

pub(crate) use component;
