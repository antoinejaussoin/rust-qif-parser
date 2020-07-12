use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct QifParsingError {
  details: String,
}

impl QifParsingError {
  pub fn new(msg: &str) -> QifParsingError {
    QifParsingError {
      details: msg.to_string(),
    }
  }
}

impl fmt::Display for QifParsingError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.details)
  }
}

impl Error for QifParsingError {
  fn description(&self) -> &str {
    &self.details
  }
}
