use super::errors::QifParsingError;
use chrono::prelude::*;

pub fn parse_date(date: &str, date_format: &str) -> Result<String, QifParsingError> {
  match NaiveDate::parse_from_str(date, date_format) {
    Err(err) => {
      let msg = format!("Error when parsing date: {} {}", err, date.to_owned());
      Err(QifParsingError::new(&msg))
    }
    Ok(d) => Ok(d.format("%Y-%m-%d").to_string()),
  }
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
}
