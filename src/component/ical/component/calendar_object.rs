use crate::{
    ContentLineParser,
    component::{
        Component, ComponentMut, IcalEvent, IcalEventBuilder, IcalJournal, IcalJournalBuilder,
        IcalTimeZone, IcalTimeZoneBuilder, IcalTodo, IcalTodoBuilder,
    },
    generator::Emitter,
    parser::{ContentLine, GetTzid, ICalProperty, ParserError, ParserOptions},
    property::{IcalCALSCALEProperty, IcalPRODIDProperty, IcalVERSIONProperty},
    types::CalDateTime,
};
use chrono::{DateTime, Utc};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt::Write,
};

#[derive(Debug, Clone)]
pub enum CalendarInnerData {
    Event(IcalEvent, Vec<IcalEvent>),
    Todo(IcalTodo, Vec<IcalTodo>),
    Journal(IcalJournal, Vec<IcalJournal>),
}

#[derive(Debug, Clone)]
pub enum CalendarInnerDataBuilder {
    Event(Vec<IcalEventBuilder>),
    Todo(Vec<IcalTodoBuilder>),
    Journal(Vec<IcalJournalBuilder>),
}

impl CalendarInnerDataBuilder {
    pub fn get_tzids(&self) -> impl Iterator<Item = &str> {
        let iterator: Box<dyn Iterator<Item = &str>> = match self {
            Self::Event(events) => Box::new(events.iter().flat_map(Component::get_tzids)),
            Self::Todo(todos) => Box::new(todos.iter().flat_map(Component::get_tzids)),
            Self::Journal(journals) => Box::new(journals.iter().flat_map(Component::get_tzids)),
        };
        iterator
    }
}

impl CalendarInnerData {
    pub fn get_uid(&self) -> &str {
        match self {
            Self::Event(main, _) => main.get_uid(),
            Self::Journal(main, _) => main.get_uid(),
            Self::Todo(main, _) => main.get_uid(),
        }
    }

    pub fn get_tzids(&self) -> impl Iterator<Item = &str> {
        let iterator: Box<dyn Iterator<Item = &str>> = match self {
            Self::Event(main, overrides) => Box::new(
                main.get_tzids()
                    .chain(overrides.iter().flat_map(Component::get_tzids)),
            ),
            Self::Todo(main, overrides) => Box::new(
                main.get_tzids()
                    .chain(overrides.iter().flat_map(Component::get_tzids)),
            ),
            Self::Journal(main, overrides) => Box::new(
                main.get_tzids()
                    .chain(overrides.iter().flat_map(Component::get_tzids)),
            ),
        };
        iterator
    }

    pub fn get_first_occurence(&self) -> Option<CalDateTime> {
        match self {
            Self::Event(main, overrides) => std::iter::once(&main.dtstart.0)
                .chain(overrides.iter().map(|over| &over.dtstart.0))
                .min(),
            Self::Todo(main, overrides) => std::iter::once(main.dtstart.as_ref().map(|dt| &dt.0))
                .chain(std::iter::once(main.due.as_ref().map(|dt| &dt.0)))
                .chain(overrides.iter().flat_map(|over| {
                    [
                        over.dtstart.as_ref().map(|dt| &dt.0),
                        over.due.as_ref().map(|dt| &dt.0),
                    ]
                }))
                .flatten()
                .min(),
            Self::Journal(main, overrides) => {
                std::iter::once(main.dtstart.as_ref().map(|dt| &dt.0))
                    .chain(
                        overrides
                            .iter()
                            .map(|over| over.dtstart.as_ref().map(|dt| &dt.0)),
                    )
                    .flatten()
                    .min()
            }
        }
        .cloned()
        .map(Into::into)
    }

    pub fn get_last_occurence(&self) -> Option<CalDateTime> {
        todo!()
    }

    pub fn from_events(mut events: Vec<IcalEvent>) -> Result<Self, ParserError> {
        let main_idx = events
            .iter()
            .position(|obj| obj.get_recurid().is_none())
            .unwrap_or_default();
        let main = events.remove(main_idx);
        if events.iter().any(|o| o.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        let overrides = events;
        if overrides.iter().any(|e| e.recurid.is_none()) {
            return Err(ParserError::MissingRecurId);
        }
        if overrides.iter().any(|e| e.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        Ok(Self::Event(main, overrides))
    }

    pub fn from_todos(mut todos: Vec<IcalTodo>) -> Result<Self, ParserError> {
        let main_idx = todos
            .iter()
            .position(|obj| obj.get_recurid().is_none())
            .unwrap_or_default();
        let main = todos.remove(main_idx);
        if todos.iter().any(|o| o.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        let overrides = todos;
        if overrides.iter().any(|e| e.recurid.is_none()) {
            return Err(ParserError::MissingRecurId);
        }
        if overrides.iter().any(|e| e.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        Ok(Self::Todo(main, overrides))
    }

    pub fn from_journals(mut journals: Vec<IcalJournal>) -> Result<Self, ParserError> {
        let main_idx = journals
            .iter()
            .position(|obj| obj.get_recurid().is_none())
            .unwrap_or_default();
        let main = journals.remove(main_idx);
        if journals.iter().any(|o| o.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        let overrides = journals;
        if overrides.iter().any(|e| e.recurid.is_none()) {
            return Err(ParserError::MissingRecurId);
        }
        if overrides.iter().any(|e| e.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        Ok(Self::Journal(main, overrides))
    }
}

impl CalendarInnerDataBuilder {
    pub fn build(
        self,
        options: &ParserOptions,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<CalendarInnerData, ParserError> {
        match self {
            Self::Event(events) => {
                let events = events
                    .into_iter()
                    .map(|builder| builder.build(options, timezones))
                    .collect::<Result<Vec<_>, _>>()?;
                CalendarInnerData::from_events(events)
            }
            Self::Todo(todos) => {
                let todos = todos
                    .into_iter()
                    .map(|builder| builder.build(options, timezones))
                    .collect::<Result<Vec<_>, _>>()?;
                CalendarInnerData::from_todos(todos)
            }
            Self::Journal(journals) => {
                let journals = journals
                    .into_iter()
                    .map(|builder| builder.build(options, timezones))
                    .collect::<Result<Vec<_>, _>>()?;
                CalendarInnerData::from_journals(journals)
            }
        }
    }
}

#[derive(Debug, Clone)]
/// An ICAL calendar object.
pub struct IcalCalendarObject {
    pub(crate) inner: CalendarInnerData,
    pub(crate) vtimezones: BTreeMap<String, IcalTimeZone>,
    pub(crate) timezones: HashMap<String, Option<chrono_tz::Tz>>,
    version: IcalVERSIONProperty,
    prodid: IcalPRODIDProperty,
    calscale: Option<IcalCALSCALEProperty>,
    pub other_properties: Vec<ContentLine>,
}

impl IcalCalendarObject {
    pub const fn get_inner(&self) -> &CalendarInnerData {
        &self.inner
    }

    pub fn get_uid(&self) -> &str {
        self.inner.get_uid()
    }

    pub const fn get_vtimezones(&self) -> &BTreeMap<String, IcalTimeZone> {
        &self.vtimezones
    }

    pub fn get_timezones(&self) -> &HashMap<String, Option<chrono_tz::Tz>> {
        &self.timezones
    }

    pub fn expand_recurrence(
        &self,
        _start: Option<DateTime<Utc>>,
        _end: Option<DateTime<Utc>>,
    ) -> Option<Cow<'_, Self>> {
        None
    }
}

#[derive(Debug, Clone, Default)]
/// An ICAL calendar object.
pub struct IcalCalendarObjectBuilder {
    pub inner: Option<CalendarInnerDataBuilder>,
    pub vtimezones: BTreeMap<String, IcalTimeZone>,
    pub content_lines: Vec<ContentLine>,
}

impl IcalCalendarObjectBuilder {
    pub fn new() -> Self {
        Self {
            content_lines: Vec::new(),
            vtimezones: BTreeMap::new(),
            inner: None,
        }
    }
}

impl Component for IcalCalendarObject {
    const NAME: &str = "VCALENDAR";
    type Builder = IcalCalendarObjectBuilder;

    fn get_tzids(&self) -> impl Iterator<Item = &str> {
        self.other_properties
            .iter()
            .flat_map(GetTzid::get_tzid)
            .chain(self.inner.get_tzids())
    }
}

impl Component for IcalCalendarObjectBuilder {
    const NAME: &str = "VCALENDAR";
    type Builder = Self;

    fn get_tzids(&self) -> impl Iterator<Item = &str> {
        std::iter::empty().chain(
            self.inner
                .iter()
                .flat_map(CalendarInnerDataBuilder::get_tzids),
        )
    }
}

impl ComponentMut for IcalCalendarObjectBuilder {
    type Verified = IcalCalendarObject;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        self.content_lines.push(content_line);
        Ok(())
    }

    fn add_sub_component<'a, I: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        line_parser: &mut ContentLineParser<'a, I>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        match value {
            IcalEvent::NAME => {
                let event = IcalEventBuilder::from_parser(line_parser, options)?;
                match &mut self.inner {
                    Some(CalendarInnerDataBuilder::Event(events)) => {
                        events.push(event);
                    }
                    None => {
                        self.inner = Some(CalendarInnerDataBuilder::Event(vec![event]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            IcalTodo::NAME => {
                let todo = IcalTodoBuilder::from_parser(line_parser, options)?;
                match &mut self.inner {
                    Some(CalendarInnerDataBuilder::Todo(todos)) => {
                        todos.push(todo);
                    }
                    None => {
                        self.inner = Some(CalendarInnerDataBuilder::Todo(vec![todo]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            IcalJournal::NAME => {
                let journal = IcalJournalBuilder::from_parser(line_parser, options)?;
                match &mut self.inner {
                    Some(CalendarInnerDataBuilder::Journal(journals)) => {
                        journals.push(journal);
                    }
                    None => {
                        self.inner = Some(CalendarInnerDataBuilder::Journal(vec![journal]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            IcalTimeZone::NAME => {
                let timezone =
                    IcalTimeZoneBuilder::from_parser(line_parser, options)?.build(options, None)?;
                self.vtimezones
                    .insert(timezone.get_tzid().to_owned(), timezone);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        options: &ParserOptions,
        _: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        let mut vtimezones: BTreeMap<String, IcalTimeZone> = self.vtimezones;
        let inner = self.inner.ok_or(ParserError::NotComplete)?;

        let mut timezones = HashMap::from_iter(
            vtimezones
                .iter()
                .map(|(name, value)| (name.clone(), value.into())),
        );

        if options.rfc7809 {
            // Populate our map of chrono timezones with those we can populate ourselves
            use std::str::FromStr;
            for tzid in inner.get_tzids() {
                if let Ok(tz) = chrono_tz::Tz::from_str(tzid) {
                    timezones.insert(tzid.to_owned(), Some(tz));
                }
            }
        }
        let inner = inner.build(options, Some(&timezones))?;
        if options.rfc7809 {
            for tzid in inner.get_tzids() {
                if !vtimezones.contains_key(tzid)
                    && let Some(tz) = IcalTimeZone::from_tzid(tzid)
                    && let Some(start) = inner.get_first_occurence()
                {
                    // Just to be safe
                    let trunc_start = start.utc() - chrono::Duration::days(365);
                    let tz = tz.clone().truncate(trunc_start);
                    vtimezones.insert(tzid.to_owned(), tz);
                }
            }
        }

        let mut version: Option<IcalVERSIONProperty> = None;
        let mut calscale: Option<IcalCALSCALEProperty> = None;
        let mut prodid: Option<IcalPRODIDProperty> = None;
        let mut other_properties = Vec::new();

        for content_line in self.content_lines {
            let property_name = content_line.name.as_str();
            match property_name {
                IcalPRODIDProperty::NAME => {
                    prodid = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalCALSCALEProperty::NAME => {
                    calscale = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                IcalVERSIONProperty::NAME => {
                    version = Some(ICalProperty::parse_prop(content_line, None)?);
                }
                _ => other_properties.push(content_line),
            }
        }

        Ok(IcalCalendarObject {
            prodid: prodid.ok_or(ParserError::MissingProperty(IcalPRODIDProperty::NAME))?,
            calscale,
            version: version.ok_or(ParserError::MissingProperty(IcalVERSIONProperty::NAME))?,
            other_properties,
            vtimezones,
            inner,
            timezones,
        })
    }
}

impl Emitter for CalendarInnerData {
    fn generate<W: Write>(&self, w: &mut W) -> std::fmt::Result {
        match self {
            Self::Event(main, overrides) => {
                main.generate(w)?;
                for over in overrides {
                    over.generate(w)?;
                }
            }
            Self::Todo(main, overrides) => {
                main.generate(w)?;
                for over in overrides {
                    over.generate(w)?;
                }
            }
            Self::Journal(main, overrides) => {
                main.generate(w)?;
                for over in overrides {
                    over.generate(w)?;
                }
            }
        }
        Ok(())
    }
}

impl Emitter for IcalCalendarObject {
    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        w.write_str("BEGIN:")?;
        w.write_str(Self::NAME)?;
        w.write_str("\r\n")?;
        self.version.generate(w)?;
        self.prodid.generate(w)?;
        self.calscale.generate(w)?;
        for property in &self.other_properties {
            property.generate(w)?;
        }
        for vtimezone in self.vtimezones.values() {
            vtimezone.generate(w)?;
        }
        self.inner.generate(w)?;
        w.write_str("END:")?;
        w.write_str(Self::NAME)?;
        w.write_str("\r\n")?;
        Ok(())
    }
}
