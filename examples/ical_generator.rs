use std::fs::read_to_string;

fn main() {
    let buf = read_to_string("./tests/resources/ical_input.ics").unwrap();
    let reader = caldata::IcalParser::from_slice(buf.as_bytes());

    for line in reader {
        println!("{:?}", &line);
        match &line {
            Err(_) => {}
            Ok(ical) => {
                let ev = ical as &dyn caldata::generator::Emitter;
                println!("{}", ev.generate());
            }
        }
    }
}
