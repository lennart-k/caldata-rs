use crate::generator::Emitter;
use crate::parser::ContentLine;
use crate::{PARAM_DELIMITER, VALUE_DELIMITER};

impl Emitter for ContentLine {
    fn generate(&self) -> String {
        let mut output = self.name.to_owned();
        if !self.params.is_empty() {
            output.push(PARAM_DELIMITER);
            output.push_str(&self.params.generate());
        }
        output.push(VALUE_DELIMITER);
        output.push_str(&self.value);
        split_line(output)
    }
}

pub(crate) fn split_line(line: String) -> String {
    let break_estimate = line.len().div_ceil(74);
    let mut output = String::with_capacity(line.len() + 3 * break_estimate + 2);

    let mut chars = line.char_indices().map(|(offset, _)| offset).peekable();
    let mut first_char_idx = 0;
    // Iterate over lines
    loop {
        // Find start of next line and find out if it was the last one
        let (line_boundary, last_line) = {
            let mut line_len = if first_char_idx == 0 { 0 } else { 1 };
            loop {
                let Some(_) = chars.next() else {
                    // We are at the end, the boundary is given bv the line length (since we don't
                    // know how wide the last character is)
                    break (line.len(), true);
                };
                line_len += 1;

                // A line should SHOULD NOT be longer than 75 characters
                if line_len == 75 {
                    // We've reached our desired length.
                    // We peek for the line boundary
                    // char_idx currently is the start of the last character
                    if let Some(&boundary) = chars.peek() {
                        break (boundary, false);
                    } else {
                        break (line.len(), true);
                    };
                }
            }
        };

        if first_char_idx == line_boundary {
            // There were no new characters
            break;
        }

        // This will not panic
        let left = line.split_at(line_boundary).0;
        #[cfg(test)]
        assert!(first_char_idx < line_boundary);
        output.push_str(left.split_at(first_char_idx).1);
        if last_line {
            break;
        } else {
            output.push_str("\r\n ");
        }
        first_char_idx = line_boundary;
    }

    output.push_str("\r\n");
    output
}

#[allow(unused)]
mod should {
    use super::split_line;

    #[test]
    fn split_line_75() {
        let text = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa75";
        let input = text.to_owned().replace("\r\n ", "");
        assert_eq!(text.to_owned() + "\r\n", split_line(input));
    }

    #[test]
    fn split_line_multibyte() {
        let text = "a";
        assert_eq!(text, split_line(text.to_owned()).replace("\r\n", ""));
        let text = "sönderzaichän :))❗okay woow❗";
        assert_eq!(text, split_line(text.to_owned()).replace("\r\n", ""));
    }

    #[test]
    fn split_long_line() {
        let text = "The ability to return a type that is only specified by the trait it impleme\r\n \
                     n\r\n";
        assert_eq!(
            text,
            split_line(text.replace("\r\n ", "").replace("\r\n", ""))
        );
        let text = "The ability to return a type that is only specified by the trait it impleme\r\n \
                     nts is especially useful in the context closures and iterators, which we c\r\n \
                     over in Chapter 13. Closures and iterators create types that only the comp\r\n \
                     iler knows or types that are very long to specify.\r\n";
        assert_eq!(
            text,
            split_line(text.replace("\r\n ", "").replace("\r\n", ""))
        );
    }

    #[test]
    fn split_long_line_multibyte() {
        // the following text includes multibyte characters (UTF-8) at strategic places to ensure
        // split_line would panic if not multibyte aware
        let text = "DESCRIPTION:ABCDEFGHIJ\\n\\nKLMNOPQRSTUVWXYZ123456789üABCDEFGHIJKLMNOPQRS\\n\\n\r\n \
                     TUVWXYZ123456ä7890ABCDEFGHIJKLM\\n\\nNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOP\r\n \
                     QRSTUVWXöYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWX\\n\\nYZ1234567890abcdefghiÜjkl\r\n \
                     m\\nnopqrstuvwx\r\n";
        assert_eq!(
            text,
            split_line(text.replace("\r\n ", "").replace("\r\n", ""))
        );
    }
}
