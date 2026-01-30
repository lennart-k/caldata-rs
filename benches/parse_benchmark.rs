use caldata::{
    IcalParser, LineReader,
    generator::{Emitter, IcalCalendar},
    parser::{ContentLine, ICalProperty},
    property::IcalDTSTARTProperty,
    types::{CalDate, CalDateTime, PartialDate},
};
use criterion::{Criterion, criterion_group, criterion_main};

fn parse_ical() -> IcalCalendar {
    let input = include_str!("../tests/resources/ical_everything.ics");
    let reader = IcalParser::from_slice(input.as_bytes());
    reader.into_iter().next().unwrap().unwrap()
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_type");
    group.bench_function("parse PartialDate", |b| {
        b.iter(|| {
            PartialDate::parse("--0329").unwrap();
        })
    });
    group.bench_function("parse CalDate", |b| {
        b.iter(|| {
            CalDate::parse("19700329", None).unwrap();
        })
    });
    group.bench_function("parse CalDateTime UTC", |b| {
        b.iter(|| {
            CalDateTime::parse("19700329T020000Z", None).unwrap();
        })
    });
    group.bench_function("parse CalDateTime Local", |b| {
        b.iter(|| {
            CalDateTime::parse("19700329T020000", None).unwrap();
        })
    });
    group.bench_function("ics parse DTSTART", |b| {
        b.iter(|| {
            let content_line = ContentLine {
                name: "DTSTART".to_owned(),
                value: "19700329T020000Z".to_owned(),
                params: vec![].into(),
            };
            IcalDTSTARTProperty::parse_prop(&content_line, None).unwrap();
        })
    });
    drop(group);
    let mut group = c.benchmark_group("lines");
    group.bench_function("line parse ical_everything.ics", |b| {
        b.iter(|| {
            let input = include_str!("../tests/resources/ical_everything.ics");
            let reader = LineReader::from_slice(input.as_bytes());
            // Consume reader
            for _ in reader {}
        })
    });
    drop(group);
    let mut group = c.benchmark_group("comps_parse");
    group.bench_function("ics parse ical_everything.ics", |b| b.iter(parse_ical));

    drop(group);
    let mut group = c.benchmark_group("comps_serialise");
    let cal = parse_ical();
    group.bench_function("ics serialise ical_everything.ics", |b| {
        b.iter(|| cal.generate())
    });
    // #[cfg(feature = "rkyv")]
    // c.bench_function("rkyv serialise ical_everything.ics", |b| {
    //     b.iter(|| rkyv::to_bytes::<rkyv::rancor::Error>(&cal).unwrap())
    // });

    // let rkyv_bytes = include_bytes!("ical_everything.rkyv");
    // #[cfg(feature = "rkyv")]
    // c.bench_function("rkyv deserialise ical_everything.ics", |b| {
    //     b.iter(|| {
    //         use ical::parser::ical::component::ArchivedIcalCalendar;
    //
    //         let archived =
    //             rkyv::access::<ArchivedIcalCalendar, rkyv::rancor::Error>(rkyv_bytes).unwrap();
    //         rkyv::deserialize::<_, rkyv::rancor::Error>(archived).unwrap();
    //     })
    // });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
