use itertools::Itertools;

use crate::{
    CalendarInnerData, IcalAlarm, IcalAlarmBuilder, IcalCalendarObject, IcalEvent,
    IcalEventBuilder, IcalFreeBusy, IcalFreeBusyBuilder, IcalJournal, IcalJournalBuilder,
    IcalTimeZone, IcalTimeZoneBuilder, IcalTodo, IcalTodoBuilder, ParserError,
    component::{Component, ComponentMut},
    generator::Emitter,
    parser::{ContentLine, ICalProperty, ParserOptions},
    property::{
        Calscale, IcalCALSCALEProperty, IcalPRODIDProperty, IcalVERSIONProperty, IcalVersion,
    },
};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct IcalCalendar {
    pub version: IcalVERSIONProperty,
    pub prodid: IcalPRODIDProperty,
    pub calscale: Option<IcalCALSCALEProperty>,
    pub other_properties: Vec<ContentLine>,
    pub event: Vec<IcalEvent>,
    pub alarm: Vec<IcalAlarm>,
    pub todo: Vec<IcalTodo>,
    pub journal: Vec<IcalJournal>,
    pub free_busy: Vec<IcalFreeBusy>,
    pub timezone: BTreeMap<String, IcalTimeZone>,
    pub chrono_timezones: HashMap<String, Option<chrono_tz::Tz>>,
}

impl Component for IcalCalendar {
    const NAME: &str = "VCALENDAR";

    type Builder = IcalCalendarBuilder;

    fn get_tzids(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
            .chain(self.event.iter().flat_map(Component::get_tzids))
            .chain(self.alarm.iter().flat_map(Component::get_tzids))
            .chain(self.todo.iter().flat_map(Component::get_tzids))
            .chain(self.journal.iter().flat_map(Component::get_tzids))
            .chain(self.free_busy.iter().flat_map(Component::get_tzids))
    }
}

#[derive(Debug, Clone, Default)]
pub struct IcalCalendarBuilder {
    pub content_lines: Vec<ContentLine>,
    pub event: Vec<IcalEventBuilder>,
    pub alarm: Vec<IcalAlarmBuilder>,
    pub todo: Vec<IcalTodoBuilder>,
    pub journal: Vec<IcalJournalBuilder>,
    pub free_busy: Vec<IcalFreeBusyBuilder>,
    pub timezone: BTreeMap<String, IcalTimeZone>,
}

impl Component for IcalCalendarBuilder {
    const NAME: &str = "VCALENDAR";

    type Builder = Self;

    fn get_tzids(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
            .chain(self.event.iter().flat_map(Component::get_tzids))
            .chain(self.alarm.iter().flat_map(Component::get_tzids))
            .chain(self.todo.iter().flat_map(Component::get_tzids))
            .chain(self.journal.iter().flat_map(Component::get_tzids))
            .chain(self.free_busy.iter().flat_map(Component::get_tzids))
    }
}

impl ComponentMut for IcalCalendarBuilder {
    type Verified = IcalCalendar;

    fn add_content_line(&mut self, content_line: ContentLine) -> Result<(), ParserError> {
        self.content_lines.push(content_line);
        Ok(())
    }

    fn add_sub_component<'a, T: Iterator<Item = std::borrow::Cow<'a, [u8]>>>(
        &mut self,
        name: &str,
        line_parser: &mut crate::ContentLineParser<'a, T>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        match name {
            IcalEvent::NAME => {
                let mut comp = IcalEvent::builder();
                comp.parse(line_parser, options)?;
                self.event.push(comp);
            }
            IcalAlarm::NAME => {
                let mut comp = IcalAlarm::builder();
                comp.parse(line_parser, options)?;
                self.alarm.push(comp);
            }
            IcalTodo::NAME => {
                let mut comp = IcalTodo::builder();
                comp.parse(line_parser, options)?;
                self.todo.push(comp);
            }
            IcalJournal::NAME => {
                let mut comp = IcalJournal::builder();
                comp.parse(line_parser, options)?;
                self.journal.push(comp);
            }
            IcalFreeBusy::NAME => {
                let mut comp = IcalFreeBusy::builder();
                comp.parse(line_parser, options)?;
                self.free_busy.push(comp);
            }
            IcalTimeZone::NAME => {
                let timezone =
                    IcalTimeZoneBuilder::from_parser(line_parser, options)?.build(options, None)?;
                self.timezone
                    .insert(timezone.get_tzid().to_owned(), timezone);
            }
            _ => return Err(ParserError::InvalidComponent(name.to_owned())),
        }
        Ok(())
    }

    fn build(
        self,
        options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        let mut timezones = HashMap::from_iter(
            self.timezone
                .iter()
                .map(|(name, value)| (name.clone(), value.into())),
        );

        let used_timezones: HashSet<&str> = std::iter::empty()
            .chain(self.event.iter().flat_map(Component::get_tzids))
            .chain(self.alarm.iter().flat_map(Component::get_tzids))
            .chain(self.todo.iter().flat_map(Component::get_tzids))
            .chain(self.journal.iter().flat_map(Component::get_tzids))
            .chain(self.free_busy.iter().flat_map(Component::get_tzids))
            .collect();

        let mut vtimezones: BTreeMap<String, IcalTimeZone> = self.timezone;
        if options.rfc7809 {
            // Populate our map of chrono timezones with those we can populate ourselves
            use std::str::FromStr;
            for tzid in used_timezones {
                if vtimezones.contains_key(tzid) {
                    continue;
                }
                if let Ok(tz) = chrono_tz::Tz::from_str(tzid)
                    && let Some(ical_tz) = IcalTimeZone::from_tzid(tzid)
                {
                    timezones.insert(tzid.to_owned(), Some(tz));
                    vtimezones.insert(tzid.to_owned(), ical_tz.clone());
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

        Ok(IcalCalendar {
            prodid: prodid.ok_or(ParserError::MissingProperty(IcalPRODIDProperty::NAME))?,
            calscale,
            version: version.ok_or(ParserError::MissingProperty(IcalVERSIONProperty::NAME))?,
            other_properties,
            event: self
                .event
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            alarm: self
                .alarm
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            todo: self
                .todo
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            journal: self
                .journal
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            free_busy: self
                .free_busy
                .into_iter()
                .map(|builder| builder.build(options, Some(&timezones)))
                .collect::<Result<_, _>>()?,
            timezone: vtimezones,
            chrono_timezones: timezones,
        })
    }
}

impl IcalCalendar {
    pub fn get_timezones(&self) -> &BTreeMap<String, IcalTimeZone> {
        &self.timezone
    }

    pub fn add_object(&mut self, object: IcalCalendarObject) {
        for (tzid, tz) in object.vtimezones.into_iter() {
            match (
                self.timezone.get(&tzid).and_then(IcalTimeZone::get_dtstart),
                tz.get_dtstart(),
            ) {
                (Some(existing), Some(new)) if new < existing => {
                    // New VTIMEZONE predates the existing one
                    self.chrono_timezones.insert(tzid.clone(), (&tz).into());
                    self.timezone.insert(tzid, tz);
                }
                (None, Some(_)) => {
                    self.chrono_timezones.insert(tzid.clone(), (&tz).into());
                    self.timezone.insert(tzid, tz);
                }
                _ => {}
            }
        }

        match object.inner {
            CalendarInnerData::Event(main, overrides) => {
                self.event.push(main);
                self.event.extend(overrides);
            }
            CalendarInnerData::Todo(main, overrides) => {
                self.todo.push(main);
                self.todo.extend(overrides);
            }
            CalendarInnerData::Journal(main, overrides) => {
                self.journal.push(main);
                self.journal.extend(overrides);
            }
        }
    }

    pub fn from_objects(
        prodid: String,
        objects: Vec<IcalCalendarObject>,
        other_properties: Vec<ContentLine>,
    ) -> Self {
        let mut cal = Self {
            version: IcalVERSIONProperty(IcalVersion::Version2_0, Default::default()),
            prodid: IcalPRODIDProperty(prodid, Default::default()),
            calscale: Some(IcalCALSCALEProperty(
                Calscale::Gregorian,
                Default::default(),
            )),
            other_properties,
            event: Vec::new(),
            alarm: Vec::new(),
            todo: Vec::new(),
            journal: Vec::new(),
            free_busy: Vec::new(),
            timezone: BTreeMap::new(),
            chrono_timezones: HashMap::new(),
        };

        for object in objects {
            cal.add_object(object);
        }

        cal
    }

    pub fn into_objects(self) -> Result<Vec<IcalCalendarObject>, ParserError> {
        let mut out = Vec::new();

        let event_groups: Vec<Vec<IcalEvent>> = self
            .event
            .into_iter()
            .chunk_by(|event| event.get_uid().to_owned())
            .into_iter()
            .map(|(_, events)| events.collect())
            .collect();

        for events in event_groups {
            let tzids: HashSet<_> = events
                .iter()
                .flat_map(|e| e.get_tzids())
                .map(ToOwned::to_owned)
                .collect();
            let inner = CalendarInnerData::from_events(events)?;
            out.push(IcalCalendarObject {
                version: self.version.clone(),
                calscale: self.calscale.clone(),
                prodid: self.prodid.clone(),
                other_properties: Vec::new(),
                vtimezones: self
                    .timezone
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.clone()))
                    .collect(),
                timezones: self
                    .chrono_timezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.to_owned()))
                    .collect(),
                inner,
            });
        }

        let journal_groups: Vec<Vec<IcalJournal>> = self
            .journal
            .into_iter()
            .chunk_by(|journal| journal.get_uid().to_owned())
            .into_iter()
            .map(|(_, journals)| journals.collect())
            .collect();

        for journals in journal_groups {
            let tzids: HashSet<_> = journals
                .iter()
                .flat_map(|e| e.get_tzids())
                .map(ToOwned::to_owned)
                .collect();
            let inner = CalendarInnerData::from_journals(journals)?;
            out.push(IcalCalendarObject {
                version: self.version.clone(),
                calscale: self.calscale.clone(),
                prodid: self.prodid.clone(),
                other_properties: Vec::new(),
                vtimezones: self
                    .timezone
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.clone()))
                    .collect(),
                timezones: self
                    .chrono_timezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.to_owned()))
                    .collect(),
                inner,
            });
        }

        let todo_groups: Vec<Vec<IcalTodo>> = self
            .todo
            .into_iter()
            .chunk_by(|todo| todo.get_uid().to_owned())
            .into_iter()
            .map(|(_, todos)| todos.collect())
            .collect();

        for todos in todo_groups {
            let tzids: HashSet<_> = todos
                .iter()
                .flat_map(|e| e.get_tzids())
                .map(ToOwned::to_owned)
                .collect();
            let inner = CalendarInnerData::from_todos(todos)?;
            out.push(IcalCalendarObject {
                version: self.version.clone(),
                calscale: self.calscale.clone(),
                prodid: self.prodid.clone(),
                other_properties: Vec::new(),
                vtimezones: self
                    .timezone
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.clone()))
                    .collect(),
                timezones: self
                    .chrono_timezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.to_owned()))
                    .collect(),
                inner,
            });
        }
        Ok(out)
    }
}

impl Emitter for IcalCalendar {
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
        for timezone in self.timezone.values() {
            timezone.generate(w)?;
        }
        self.event.generate(w)?;
        self.alarm.generate(w)?;
        self.todo.generate(w)?;
        self.journal.generate(w)?;
        self.free_busy.generate(w)?;
        w.write_str("END:")?;
        w.write_str(Self::NAME)?;
        w.write_str("\r\n")?;
        Ok(())
    }
}
