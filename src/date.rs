use super::errors::QifParsingError;
use chrono::prelude::*;

pub fn parse_date(date: &str, date_format: &str) -> Result<String, QifParsingError> {
    let date = date.trim();
    match try_parse(date, date_format) {
        Ok(parsed) => Ok(parsed),
        Err(err) => {
            // Quicken pads a single-digit month or day with a space: "6/ 1/94", "1/ 1/10".
            // Retry without those spaces when the caller's format has no room for them.
            // A space that is not immediately after a slash (the "' 7" year in "12/21' 7")
            // is left alone, so an explicit format can still match it on the first attempt.
            let normalized = strip_slash_padding(date);
            if normalized != date
                && let Ok(parsed) = try_parse(&normalized, date_format)
            {
                return Ok(parsed);
            }
            let msg = format!("Error when parsing date: {err} {date}");
            Err(QifParsingError::new(&msg))
        }
    }
}

fn try_parse(date: &str, date_format: &str) -> Result<String, chrono::ParseError> {
    let parsed = NaiveDate::parse_from_str(date, date_format)?;
    Ok(parsed.format("%Y-%m-%d").to_string())
}

/// Choose one chrono format that accepts every date in `samples`.
///
/// A number above 12 fixes the order: it cannot be a month. When every date
/// can be read either way, the guess is month-first, which is Quicken's own
/// default. A `'` before the year selects the apostrophe forms (`12/21' 7`,
/// `2/10'2020`). Two-digit and four-digit years are not mixed.
pub(crate) fn guess_format(samples: &[&str]) -> Result<&'static str, QifParsingError> {
    let shapes: Vec<DateShape> = samples
        .iter()
        .filter_map(|sample| date_shape(sample))
        .collect();
    if shapes.is_empty() {
        return Ok("%m/%d/%Y");
    }
    if shapes
        .iter()
        .any(|shape| !matches!(shape.year_len, 1 | 2 | 4))
    {
        return Err(QifParsingError::new(
            "Could not guess the date format: a year is not 2 or 4 digits",
        ));
    }
    let apostrophe = shapes[0].apostrophe;
    if shapes.iter().any(|shape| shape.apostrophe != apostrophe) {
        return Err(QifParsingError::new(
            "Could not guess the date format: the file mixes '/' and \"'\" date separators",
        ));
    }
    let long_year = shapes.iter().any(|shape| shape.year_len == 4);
    let short_year = shapes.iter().any(|shape| shape.year_len <= 2);
    if long_year && short_year {
        return Err(QifParsingError::new(
            "Could not guess the date format: the file mixes 2-digit and 4-digit years",
        ));
    }
    let day_first = shapes.iter().any(|shape| shape.first > 12);
    let month_first = shapes.iter().any(|shape| shape.second > 12);
    if day_first && month_first {
        return Err(QifParsingError::new(
            "Could not guess the date format: some dates are day-first and some are month-first",
        ));
    }
    Ok(format_for(day_first, apostrophe, long_year))
}

struct DateShape {
    first: u32,
    second: u32,
    year_len: usize,
    apostrophe: bool,
}

fn date_shape(raw: &str) -> Option<DateShape> {
    let normalized = strip_slash_padding(raw.trim());
    let (first, rest) = normalized.split_once('/')?;
    let first = first.parse().ok()?;
    let (second, year, apostrophe) = if let Some((second, year)) = rest.split_once('/') {
        (second, year, false)
    } else if let Some((second, year)) = rest.split_once('\'') {
        (second, year, true)
    } else {
        return None;
    };
    let second = second.trim().parse().ok()?;
    let year = year.trim();
    if year.is_empty() || !year.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if !(1..=31).contains(&first) || !(1..=31).contains(&second) {
        return None;
    }
    Some(DateShape {
        first,
        second,
        year_len: year.len(),
        apostrophe,
    })
}

fn format_for(day_first: bool, apostrophe: bool, long_year: bool) -> &'static str {
    match (day_first, apostrophe, long_year) {
        (true, false, true) => "%d/%m/%Y",
        (true, false, false) => "%d/%m/%y",
        (false, false, true) => "%m/%d/%Y",
        (false, false, false) => "%m/%d/%y",
        (true, true, true) => "%d/%m'%Y",
        (true, true, false) => "%d/%m'%y",
        (false, true, true) => "%m/%d'%Y",
        (false, true, false) => "%m/%d'%y",
    }
}

fn strip_slash_padding(date: &str) -> String {
    let mut out = String::with_capacity(date.len());
    let mut chars = date.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '/' {
            while matches!(chars.peek(), Some(' ')) {
                chars.next();
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_day_first_date() {
        assert_eq!(parse_date("13/01/2020", "%d/%m/%Y").unwrap(), "2020-01-13");
    }

    #[test]
    fn test_parse_month_first_date() {
        assert_eq!(parse_date("02/13/2020", "%m/%d/%Y").unwrap(), "2020-02-13");
    }

    #[test]
    fn test_unpadded_month_and_day() {
        assert_eq!(parse_date("7/8/93", "%m/%d/%y").unwrap(), "1993-07-08");
        assert_eq!(parse_date("1/1/10", "%d/%m/%y").unwrap(), "2010-01-01");
        assert_eq!(parse_date("29/12/10", "%d/%m/%y").unwrap(), "2010-12-29");
    }

    #[test]
    fn test_quicken_space_padded_day() {
        assert_eq!(parse_date("6/ 1/94", "%m/%d/%y").unwrap(), "1994-06-01");
        assert_eq!(parse_date("1/ 1/10", "%m/%d/%y").unwrap(), "2010-01-01");
        assert_eq!(parse_date("6/  1/94", "%m/%d/%y").unwrap(), "1994-06-01");
    }

    #[test]
    fn test_apostrophe_year_keeps_its_space() {
        assert_eq!(parse_date("12/21' 7", "%m/%d'%y").unwrap(), "2007-12-21");
        assert_eq!(parse_date("2/10'2020", "%m/%d'%Y").unwrap(), "2020-02-10");
    }

    #[test]
    fn test_surrounding_whitespace_is_ignored() {
        assert_eq!(
            parse_date("  13/01/2020  ", "%d/%m/%Y").unwrap(),
            "2020-01-13"
        );
    }

    #[test]
    fn test_out_of_range_message_is_stable() {
        let err = parse_date("27/08/2018", "%m/%d/%Y").unwrap_err();
        assert_eq!(
            err.details,
            "Error when parsing date: input is out of range 27/08/2018"
        );
    }

    #[test]
    fn guess_uses_a_number_above_12_to_fix_the_order() {
        assert_eq!(guess_format(&["27/08/2018"]).unwrap(), "%d/%m/%Y");
        assert_eq!(guess_format(&["02/13/2020"]).unwrap(), "%m/%d/%Y");
        assert_eq!(guess_format(&["29/12/10"]).unwrap(), "%d/%m/%y");
        assert_eq!(guess_format(&["12/29/10", "6/ 1/94"]).unwrap(), "%m/%d/%y");
        assert_eq!(guess_format(&["12/21' 7"]).unwrap(), "%m/%d'%y");
        assert_eq!(
            guess_format(&["2/14'2020", "2/10'2020"]).unwrap(),
            "%m/%d'%Y"
        );
    }

    #[test]
    fn guess_is_month_first_when_every_date_is_ambiguous() {
        assert_eq!(guess_format(&["6/1/94", "6/2/94"]).unwrap(), "%m/%d/%y");
        assert_eq!(guess_format(&["03/04/10"]).unwrap(), "%m/%d/%y");
        assert_eq!(guess_format(&["1/2/2020"]).unwrap(), "%m/%d/%Y");
    }

    #[test]
    fn guess_rejects_a_file_that_needs_two_orders() {
        let err = guess_format(&["13/01/2020", "01/13/2020"]).unwrap_err();
        assert_eq!(
            err.details,
            "Could not guess the date format: some dates are day-first and some are month-first"
        );
    }
}
