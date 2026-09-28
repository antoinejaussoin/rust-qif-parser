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
}
