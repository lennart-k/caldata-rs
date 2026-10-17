use std::borrow::Cow;

use caldata::LineReader;

// Simple function for sorting properties and components
// to allow for order-invariant comparison of Emitter outputs
pub fn lines_normalise_prop_order<'a>(
    line_iter: &mut impl Iterator<Item = Cow<'a, str>>,
    header: Option<Cow<'a, str>>,
) -> Vec<Cow<'a, str>>
where
{
    let mut props = vec![];
    let mut comps = vec![];
    let mut end = None;
    while let Some(line) = line_iter.next() {
        if line.to_uppercase().starts_with("BEGIN:") {
            comps.push(lines_normalise_prop_order(line_iter, Some(line)));
        } else if line.to_uppercase().starts_with("END:") {
            end = Some(line);
            break;
        } else {
            props.push(line);
        }
    }
    assert_eq!(header.is_some(), end.is_some());
    props.sort();

    [
        header.map(|hdr| vec![hdr]).unwrap_or_default(),
        props,
        comps.into_iter().flatten().collect(),
        end.map(|end| vec![end]).unwrap_or_default(),
    ]
    .concat()
}

pub fn str_normalise_prop_order(input: &str) -> String {
    let mut lines = LineReader::from_slice(input.as_bytes()).map(|line| line.unwrap().inner);
    let sorted = lines_normalise_prop_order(&mut lines, None);
    sorted.join("\r\n") + "\r\n"
}

#[cfg(test)]
mod tests {
    use super::{lines_normalise_prop_order, str_normalise_prop_order};
    use caldata::LineReader;
    use itertools::Itertools;
    use rstest::rstest;

    macro_rules! set_snapshot_suffix {
        ($($expr:expr),*) => {
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_suffix(format!($($expr,)*));
            let _guard = settings.bind_to_scope();
        }
    }

    #[test]
    fn test_sort_output_lines() {
        let lines = vec![
            "a",
            "c",
            "b",
            "begin:event",
            "d",
            "a",
            "begin:alarm",
            "g",
            "f",
            "end:alarm",
            "end:event",
            "begin:event",
            "p",
            "a",
            "end:event",
            "d",
        ];
        let input = lines.join("\r\n") + "\r\n";
        let mut lines = LineReader::from_slice(input.as_bytes()).map(|line| line.unwrap().inner);
        let sorted = lines_normalise_prop_order(&mut lines, None);
        assert_eq!(
            sorted.iter().collect_vec(),
            vec![
                "a",
                "b",
                "c",
                "d",
                "begin:event",
                "a",
                "d",
                "begin:alarm",
                "f",
                "g",
                "end:alarm",
                "end:event",
                "begin:event",
                "a",
                "p",
                "end:event",
            ]
        );
    }

    #[rstest]
    #[case(0, include_str!("./resources/ical_events.ics"))]
    #[case(1, include_str!("./resources/vcard_input.vcf"))]
    #[case(2, include_str!("./resources/ical_todos.ics"))]
    #[case(3, include_str!("./resources/ical_journals.ics"))]
    #[case(4, include_str!("./resources/recurring_wholeday.ics"))]
    fn test_sort_props(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        insta::assert_snapshot!(str_normalise_prop_order(input));
    }
}
