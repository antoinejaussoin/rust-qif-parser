mod date;
mod errors;

/// Represents a QIF file
/// It has a file_type (Bank, etc.) and a collection of items
pub struct Qif {
  /// File type can be one of: Cash, Bank, CCard, Invst, Oth A, Oth L, Invoice
  pub file_type: String,
  pub items: Vec<QifItem>,
}

/// Represents a transaction
/// It has a date and an amount, and possibly some splits
pub struct QifItem {
  pub date: String,
  pub amount: f32,
  pub payee: String,
  pub category: String,
  pub cleared_status: String,
  pub address: Vec<String>,
  pub splits: Vec<QifSplit>,
}

/// Represent a Split, which is basically a portion of a transaction
pub struct QifSplit {
  pub category: String,
  pub memo: String,
  pub amount: f32,
}

fn empty_item() -> QifItem {
  QifItem {
    date: "".to_string(),
    amount: 0.0,
    payee: "".to_string(),
    category: "".to_string(),
    cleared_status: "".to_string(),
    splits: Vec::new(),
    address: Vec::new(),
  }
}

/// This is the parsing function. It takes the text content of your QIF file as an argument,
/// and the date format.
///
/// Indeed, the date in a QIF file doesn't have a pre-determined format, which means you could
/// receive QIF files with completely different formats.
/// Please use, for the date_format, the format you would use with Chrono (https://docs.rs/chrono/0.4.13/chrono/format/strftime/index.html#specifiers)
///
/// The parser will then return a Qif data structure or an error
pub fn parse(qif_content: &str, date_format: &str) -> Result<Qif, errors::QifParsingError> {
  let mut results: Vec<QifItem> = Vec::new();
  let mut result = Qif {
    file_type: "".to_string(),
    items: Vec::new(),
  };
  let mut current = empty_item();
  let lines: Vec<&str> = qif_content.lines().collect();

  for line in lines {
    if line.starts_with("!Type") {
      result.file_type = line[6..].to_string();
    }
    if line.starts_with("^") {
      results.push(current);
      current = empty_item();
    }
    match parse_line(line, &mut current, date_format) {
      Err(err) => return Err(err),
      Ok(()) => (),
    }
  }

  result.items = results;

  Ok(result)
}

fn parse_number(line: &str) -> Result<f32, errors::QifParsingError> {
  match line[1..].to_string().trim().parse() {
    Err(_err) => {
      let msg = format!(
        "Could not parse the following as a number: '{}'",
        &line[1..]
      );
      Err(errors::QifParsingError::new(&msg))
    }
    Ok(amount) => Ok(amount),
  }
}

fn parse_line(
  line: &str,
  item: &mut QifItem,
  date_format: &str,
) -> Result<(), errors::QifParsingError> {
  if line.starts_with("T") || line.starts_with("U") {
    item.amount = match parse_number(line) {
      Err(err) => return Err(err),
      Ok(amount) => amount,
    };
  }
  if line.starts_with("P") {
    item.payee = line[1..].to_string();
  }
  if line.starts_with("L") {
    item.category = line[1..].to_string();
  }
  if line.starts_with("D") {
    item.date = match date::parse_date(&line[1..], date_format) {
      Err(err) => return Err(err),
      Ok(date) => date,
    };
  }
  if line.starts_with("C") {
    item.cleared_status = line[1..].to_string();
  }
  if line.starts_with("A") {
    item.address.push(line[1..].to_string());
  }

  // Split
  if line.starts_with("S") {
    let split = QifSplit {
      category: line[1..].to_string(),
      memo: "".to_string(),
      amount: 0.0,
    };
    item.splits.push(split);
  }
  if line.starts_with("E") {
    let split = match item.splits.last_mut() {
      None => {
        return Err(errors::QifParsingError::new(
          "There should be a split item here",
        ))
      }
      Some(item) => item,
    };
    split.memo = line[1..].to_string();
  }
  if line.starts_with("$") {
    let split = match item.splits.last_mut() {
      None => {
        return Err(errors::QifParsingError::new(
          "There should be a split item here",
        ))
      }
      Some(item) => item,
    };
    split.amount = match parse_number(line) {
      Err(err) => return Err(err),
      Ok(amount) => amount,
    };
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  #[test]
  fn test_wikipedia_example() {
    let content = fs::read_to_string("data/wikipedia.qif").unwrap();
    let result = parse(&content, "%m/%d'%Y").unwrap();
    assert!(content.len() > 0);
    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert_eq!(result.items.len(), 6);

    // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2020-02-10");
    assert_eq!(first.amount, 0.0);
    assert_eq!(first.category, "[TestExport]");
    assert_eq!(first.payee, "Opening Balance");
    assert_eq!(first.cleared_status, "X");

    // Second item (splits)
    let second = &result.items[1];
    assert_eq!(second.splits.len(), 2);
    assert_eq!(second.splits[0].category, "Bills:Cell Phone");
    assert_eq!(second.splits[0].memo, "sign up credit");
    assert_eq!(second.splits[0].amount, -15.0);
    assert_eq!(second.splits[1].category, "Bills:Cell Phone");
    assert_eq!(second.splits[1].memo, "new account");
    assert_eq!(second.splits[1].amount, 82.50);
  }

  #[test]
  fn test_monzo_example() {
    let content = fs::read_to_string("data/monzo.qif").unwrap();
    let result = parse(&content, "%d/%m/%Y").unwrap();
    assert!(content.len() > 0);

    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert_eq!(result.items.len(), 13);

    // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2018-08-27");
    assert_eq!(first.amount, 1000.0);
    assert_eq!(first.category, "");
    assert_eq!(first.payee, "Jane Doe");
    assert_eq!(first.cleared_status, "");

    // Third items
    let third = &result.items[2];
    assert_eq!(third.date, "2018-08-28");
    assert_eq!(third.amount, -15.0);
    assert_eq!(third.category, "Transport");
    assert_eq!(third.payee, "Infinity Motor Cycles");
    assert_eq!(third.address[0], "30-32 FairyLand High Street");
  }

  #[test]
  fn test_cic_example() {
    let content = fs::read_to_string("data/cic.qif").unwrap();
    let result = parse(&content, "%d/%m/%y").unwrap();
    assert!(content.len() > 0);

    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert_eq!(result.items.len(), 12);

    // // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2020-05-19");
    assert_eq!(first.amount, 500.0);
    assert_eq!(first.category, "");
    assert_eq!(first.payee, "REM CHQ REF1234");
    assert_eq!(first.cleared_status, "");

    // // Third items
    let third = &result.items[2];
    assert_eq!(third.date, "2020-06-02");
    assert_eq!(third.amount, -9.59);
    assert_eq!(third.category, "");
    assert_eq!(third.payee, "KIMSUFI CARTE 1234 PAIEMENT CB 0106 ROUBAIX");
    assert_eq!(third.address.len(), 0);
  }

  #[test]
  fn test_errors() {
    let content = fs::read_to_string("data/monzo.qif").unwrap();
    match parse(&content, "%m/%d/%Y") {
      Err(err) => assert_eq!(
        err.details,
        "Error when parsing date: input is out of range 27/08/2018"
      ),
      Ok(_) => panic!("It should have failed"),
    };
  }
}
