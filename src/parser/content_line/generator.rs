use std::fmt::Write;

use crate::generator::{Emitter, LineWriter};
use crate::parser::ContentLine;
use crate::{PARAM_DELIMITER, VALUE_DELIMITER};

impl Emitter for ContentLine {
    fn generate<W: Write>(&self, i: &mut W) -> std::fmt::Result {
        let mut w = LineWriter::new(i);
        w.write_str(&self.name)?;
        if !self.params.is_empty() {
            w.write_char(PARAM_DELIMITER)?;
            self.params.generate(&mut w)?;
        }
        w.write_char(VALUE_DELIMITER)?;
        w.write_str(&self.value)?;
        w.write_str("\r\n")?;
        Ok(())
    }
}
