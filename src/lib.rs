const PARAM_VALUE_DELIMITER: char = ',';
const VALUE_DELIMITER: char = ':';
const PARAM_DELIMITER: char = ';';
const PARAM_NAME_DELIMITER: char = '=';
const PARAM_QUOTE: char = '"';

pub mod component;
pub use component::ical::*;
pub use component::vcard::VcardParser;

pub mod parser;
pub use parser::{ComponentParser, ContentLineParser, LineReader, ParserError};

pub mod property;

pub mod generator;

pub mod types;
