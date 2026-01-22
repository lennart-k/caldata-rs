use std::fs::read_to_string;

fn main() {
    let buf = read_to_string("./tests/ressources/ical_input.ics").unwrap();

    let reader = caldata::PropertyParser::from_slice(buf.as_bytes());

    for line in reader {
        println!("{:?}", line);
    }
}
