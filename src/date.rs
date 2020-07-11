use chrono::prelude::*;
pub fn parse_date(date: &str, date_format: &str) -> String {
  match NaiveDate::parse_from_str(date, date_format) {
    Err(err) => {
      println!("Error when parsing date: {} {}", err, date);
      panic!()
    }
    Ok(d) => d.format("%Y-%m-%d").to_string(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_parse_day_first_date() {
    println!(
      "current dir: {}",
      std::env::current_dir().unwrap().to_str().unwrap()
    );
    assert_eq!(parse_date("13/01/2020", "%d/%m/%Y"), "2020-01-13");
  }

  #[test]
  fn test_parse_month_first_date() {
    assert_eq!(parse_date("02/13/2020", "%m/%d/%Y"), "2020-02-13");
  }

  fn test_wikipedia_example() {}
}
