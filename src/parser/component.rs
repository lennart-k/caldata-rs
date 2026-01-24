use std::{borrow::Cow, marker::PhantomData};

use crate::{
    ContentLineParser, LineReader, ParserError,
    component::{Component, ComponentMut},
    parser::{BytesLines, ParserOptions},
};

pub struct ComponentParser<'a, C: Component, I: Iterator<Item = Cow<'a, [u8]>>> {
    line_parser: ContentLineParser<'a, I>,
    _t: PhantomData<C>,
    options: ParserOptions,
}

impl<'a, C: Component> ComponentParser<'a, C, BytesLines<'a>> {
    /// Return a new `ComponentParser` from a `Reader`.
    pub fn from_slice(slice: &'a [u8]) -> Self {
        let line_reader = LineReader::from_slice(slice);
        let line_parser = ContentLineParser::new(line_reader);

        ComponentParser {
            line_parser,
            _t: Default::default(),
            options: Default::default(),
        }
    }

    pub fn with_options(mut self, options: ParserOptions) -> Self {
        self.options = options;
        self
    }
}

impl<'a, C: Component, I: Iterator<Item = Cow<'a, [u8]>>> ComponentParser<'a, C, I> {
    /// Read the next line and check if it's a valid VCALENDAR start.
    #[inline]
    fn check_header(&mut self) -> Result<Option<()>, ParserError> {
        let line = match self.line_parser.next() {
            Some(val) => val.map_err(ParserError::ContentLineError)?,
            None => return Ok(None),
        };

        if line.name != "BEGIN"
            || line.value.is_none()
            || !C::NAMES.contains(&line.value.as_ref().unwrap().to_uppercase().as_str())
            || !line.params.is_empty()
        {
            return Err(ParserError::MissingHeader);
        }

        Ok(Some(()))
    }

    pub fn expect_one(mut self) -> Result<<C::Unverified as ComponentMut>::Verified, ParserError> {
        let item = self.next().ok_or(ParserError::EmptyInput)??;
        if self.next().is_some() {
            return Err(ParserError::TooManyComponents);
        }
        Ok(item)
    }
}

impl<'a, C: Component, I: Iterator<Item = Cow<'a, [u8]>>> Iterator for ComponentParser<'a, C, I> {
    type Item = Result<<C::Unverified as ComponentMut>::Verified, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.check_header() {
            Ok(res) => res?,
            Err(err) => return Some(Err(err)),
        };

        let mut comp = C::Unverified::default();
        let result = match comp.parse(&mut self.line_parser, &self.options) {
            Ok(_) => comp.build(None),
            Err(err) => Err(err),
        };

        #[cfg(all(feature = "test", not(feature = "bench")))]
        {
            // Run this for more test coverage
            if let Ok(comp) = result.as_ref() {
                let mutable = comp.clone().mutable();
                mutable.get_properties();
            }
        }

        Some(result)
    }
}
