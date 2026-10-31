mod ical;
use std::collections::BTreeMap;
use std::fmt::Write;

pub use crate::component::ical::component::{IcalCalendar, IcalEvent};
pub use crate::component::vcard::component::VcardContact;
pub use crate::parser::ContentLine;

///
/// Emits the content of the Component in ical-format.
///
pub trait Emitter {
    /// creates a textual-representation of this object and all it's properties
    /// in ical-format.
    fn generate<W: Write>(&self, w: &mut W) -> std::fmt::Result;

    fn generate_string(&self) -> String {
        let mut out = String::new();
        self.generate(&mut out)
            .expect("Formatting to a string is infallible");
        out
    }
}

impl<K, T: Emitter> Emitter for BTreeMap<K, T> {
    fn generate<W: Write>(&self, w: &mut W) -> std::fmt::Result {
        for component in self.values() {
            component.generate(w)?;
        }
        Ok(())
    }
}

impl<T: Emitter> Emitter for Vec<T> {
    fn generate<W: Write>(&self, w: &mut W) -> std::fmt::Result {
        for component in self {
            component.generate(w)?;
        }
        Ok(())
    }
}

const MAX_LINE_LEN: usize = 75;

pub struct LineWriter<'w, W: Write> {
    writer: &'w mut W,
    current_line_len: usize,
}

impl<'w, W: Write> LineWriter<'w, W> {
    pub fn new(writer: &'w mut W) -> Self {
        Self {
            writer,
            current_line_len: 0,
        }
    }

    fn break_line(&mut self) -> std::fmt::Result {
        self.writer.write_str("\r\n ")?;
        self.current_line_len = 1;
        Ok(())
    }
}

impl<'w, W: Write> Write for LineWriter<'w, W> {
    fn write_str(&mut self, mut rest: &str) -> std::fmt::Result {
        while !rest.is_empty() {
            let remaining_capacity = MAX_LINE_LEN - self.current_line_len;
            if remaining_capacity == 0 {
                self.break_line()?;
            }

            let (byte_idx, char_count) = rest
                // keep track of byte index
                .char_indices()
                // keep track of character index
                .enumerate()
                .take(remaining_capacity)
                .last()
                .map(|(i_char, (i_byte, chr))| (i_byte + chr.len_utf8(), i_char + 1))
                .unwrap_or((0, 0));

            let to_write;
            (to_write, rest) = rest.split_at(byte_idx);

            self.writer.write_str(to_write)?;
            self.current_line_len += char_count;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> std::fmt::Result {
        if self.current_line_len >= MAX_LINE_LEN {
            self.break_line()?;
        }
        self.writer.write_char(c)?;
        self.current_line_len += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::LineWriter;

    fn split_line(text: &str) -> String {
        let mut out = String::new();
        let mut writer = LineWriter::new(&mut out);
        writer.write_str(text).unwrap();
        out
    }

    #[rstest::rstest]
    #[case("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa75")]
    #[case("a")]
    #[case("sönderzaichän :))❗okay woow❗")]
    fn split_line_unchanged(#[case] input: &str) {
        let mut out = String::new();
        let mut writer = LineWriter::new(&mut out);
        writer.write_str(input).unwrap();
        assert_eq!(input, out);
    }

    #[test]
    fn split_long_line() {
        let text = "The ability to return a type that is only specified by the trait it impleme\r\n \
                     n\r\n";
        assert_eq!(text, split_line(&text.replace("\r\n ", "")));
        let text = "The ability to return a type that is only specified by the trait it impleme\r\n \
                     nts is especially useful in the context closures and iterators, which we c\r\n \
                     over in Chapter 13. Closures and iterators create types that only the comp\r\n \
                     iler knows or types that are very long to specify.\r\n";
        assert_eq!(text, split_line(&text.replace("\r\n ", "")));
    }

    #[test]
    fn split_long_line_multibyte() {
        // the following text includes multibyte characters (UTF-8) at strategic places to ensure
        // split_line would panic if not multibyte aware
        let text = "DESCRIPTION:ABCDEFGHIJ\\n\\nKLMNOPQRSTUVWXYZ123456789üABCDEFGHIJKLMNOPQRS\\n\\n\r\n \
                     TUVWXYZ123456ä7890ABCDEFGHIJKLM\\n\\nNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOP\r\n \
                     QRSTUVWXöYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWX\\n\\nYZ1234567890abcdefghiÜjkl\r\n \
                     m\\nnopqrstuvwx\r\n";
        assert_eq!(text, split_line(&text.replace("\r\n ", "")));
    }
}
