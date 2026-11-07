use crate::{
    ContentLineParser,
    component::{
        CalendarInnerData, Component, ComponentMut, IcalAlarm, IcalAlarmBuilder,
        IcalCalendarObject, IcalEvent, IcalEventBuilder, IcalFreeBusy, IcalFreeBusyBuilder,
        IcalJournal, IcalJournalBuilder, IcalTimeZone, IcalTodo, IcalTodoBuilder, ParserError,
    },
    generator::Emitter,
    parser::{ContentLine, ParserOptions},
    property::{
        Calscale, IcalCALSCALEProperty, IcalPRODIDProperty, IcalVERSIONProperty, IcalVersion,
    },
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
};

#[derive(Debug, Clone, Default)]
/// An ICAL calendar.
pub struct IcalCalendar<
    const VERIFIED: bool = true,
    A = IcalAlarm,
    E = IcalEvent,
    F = IcalFreeBusy,
    J = IcalJournal,
    T = IcalTodo,
> {
    pub properties: Vec<ContentLine>,
    pub events: Vec<E>,
    pub alarms: Vec<A>,
    pub todos: Vec<T>,
    pub journals: Vec<J>,
    pub free_busys: Vec<F>,
    pub vtimezones: BTreeMap<String, IcalTimeZone>,
    pub timezones: HashMap<String, Option<chrono_tz::Tz>>,
}
pub type IcalCalendarBuilder = IcalCalendar<
    false,
    IcalAlarmBuilder,
    IcalEventBuilder,
    IcalFreeBusyBuilder,
    IcalJournalBuilder,
    IcalTodoBuilder,
>;

impl Component for IcalCalendar {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Builder = IcalCalendarBuilder;
}

impl Component for IcalCalendarBuilder {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Builder = IcalCalendarBuilder;
}

impl ComponentMut for IcalCalendarBuilder {
    type Verified = IcalCalendar;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        self.properties.push(content_line);
        Ok(())
    }

    #[inline]
    fn add_sub_component<'a, I: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        line_parser: &mut ContentLineParser<'a, I>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                self.alarms
                    .push(IcalAlarmBuilder::from_parser(line_parser, options)?);
            }
            "VEVENT" => {
                self.events
                    .push(IcalEventBuilder::from_parser(line_parser, options)?);
            }
            "VTODO" => {
                self.todos
                    .push(IcalTodoBuilder::from_parser(line_parser, options)?);
            }
            "VJOURNAL" => {
                self.journals
                    .push(IcalJournalBuilder::from_parser(line_parser, options)?);
            }
            "VFREEBUSY" => {
                self.free_busys
                    .push(IcalFreeBusyBuilder::from_parser(line_parser, options)?);
            }
            "VTIMEZONE" => {
                let timezone =
                    IcalTimeZone::from_parser(line_parser, options)?.build(options, None)?;
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
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        let _version: IcalVERSIONProperty = todo!();
        // This should technically be REQUIRED but Apple Calendar doesn't adhere to the spec. :(
        let _prodid: Option<IcalPRODIDProperty> = None;
        let _calscale: Option<IcalCALSCALEProperty> = None;

        #[allow(unused_mut)]
        let mut timezones = HashMap::from_iter(
            self.vtimezones
                .iter()
                .map(|(tzid, tz)| (tzid.to_owned(), tz.into())),
        );

        let used_tzids: HashSet<&str> = self
            .events
            .iter()
            .flat_map(|object| vec![])
            .chain(self.alarms.iter().flat_map(|object| object.get_tzids()))
            .chain(self.todos.iter().flat_map(|object| object.get_tzids()))
            .chain(self.journals.iter().flat_map(|object| object.get_tzids()))
            .chain(self.free_busys.iter().flat_map(|object| object.get_tzids()))
            .collect();

        let mut vtimezones = self.vtimezones;
        if options.rfc7809 {
            // Populate our map of chrono timezones with those we can populate ourselves
            use std::str::FromStr;
            for tzid in used_tzids {
                if let Ok(tz) = chrono_tz::Tz::from_str(tzid)
                    && let Some(ical_tz) = IcalTimeZone::from_tzid(tzid)
                {
                    timezones.insert(tzid.to_owned(), Some(tz));
                    vtimezones.insert(tzid.to_owned(), ical_tz.clone());
                }
            }
        }

        Ok(IcalCalendar {
            properties: self.properties,
            events: self
                .events
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            alarms: self
                .alarms
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            todos: self
                .todos
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            journals: self
                .journals
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            free_busys: self
                .free_busys
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            vtimezones,
            timezones,
        })
    }
}

impl IcalCalendar {
    pub fn from_objects(
        prodid: String,
        objects: Vec<IcalCalendarObject>,
        additional_properties: Vec<ContentLine>,
    ) -> Self {
        let mut cal = IcalCalendar {
            events: vec![],
            todos: vec![],
            journals: vec![],
            alarms: vec![],
            free_busys: vec![],
            properties: vec![
                IcalVERSIONProperty(IcalVersion::Version2_0, vec![].into()).into(),
                ContentLine {
                    name: "PRODID".to_owned(),
                    value: prodid,
                    params: Default::default(),
                },
                IcalCALSCALEProperty(Calscale::Gregorian, vec![].into()).into(),
            ],
            vtimezones: BTreeMap::new(),
            timezones: HashMap::new(),
        };
        cal.properties.extend_from_slice(&additional_properties);
        for object in objects {
            object.add_to_calendar(&mut cal);
        }
        cal
    }

    pub fn into_objects(self) -> Result<Vec<IcalCalendarObject>, ParserError> {
        todo!()
    }
}

impl Emitter for IcalCalendar {
    fn generate<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        Ok(())
    }
}
