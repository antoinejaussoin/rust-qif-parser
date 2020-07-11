use crate::date;

pub struct Qif {
  pub file_type: String,
  pub items: Vec<QifItem>,
}

pub struct QifItem {
  pub date: String,
  pub amount: f32,
  pub payee: String,
  pub category: String,
  pub cleared_status: String,
  pub splits: Vec<QifSplit>,
}

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
  }
}

pub fn parse_with_format(qif_content: &str, date_format: &str) -> Qif {
  let mut results: Vec<QifItem> = Vec::new();
  let mut result = Qif {
    file_type: "".to_string(),
    items: Vec::new(),
  };
  let mut current = empty_item();
  let lines: Vec<&str> = qif_content.split("\n").collect();
  println!("We've got {} lines", lines.len());

  for line in lines {
    if line.starts_with("!Type") {
      result.file_type = line[6..].to_string();
    }
    if line.starts_with("^") {
      results.push(current);
      current = empty_item();
    }
    parse_line(line, &mut current, date_format);
  }

  result.items = results;

  result
}

// pub fn parse(qif_content: &str) -> Qif {
//   parse_with_format(qif_content, "%d/%m/%Y")
// }

fn parse_line(line: &str, item: &mut QifItem, date_format: &str) {
  if line.starts_with("T") || line.starts_with("U") {
    let amount: f32 = match line[1..].to_string().trim().parse() {
      Err(err) => {
        println!("Error: {} '{}'", err, line[1..].to_string().trim());
        0.0
      }
      Ok(amount) => amount,
    };
    item.amount = amount;
  }
  if line.starts_with("P") {
    item.payee = line[1..].to_string();
  }
  if line.starts_with("L") {
    item.category = line[1..].to_string();
  }
  if line.starts_with("D") {
    item.date = date::parse_date(&line[1..], date_format);
  }
  if line.starts_with("C") {
    item.cleared_status = line[1..].to_string();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  #[test]
  fn test_wikipedia_example() {
    let content = fs::read_to_string("data/example1.qif").unwrap();
    let result = parse_with_format(&content, "%m/%d'%Y");
    assert!(content.len() > 0);
    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert!(result.items.len() == 6);

    // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2020-02-10");
    assert_eq!(first.amount, 0.0);
    assert_eq!(first.category, "[TestExport]");
    assert_eq!(first.payee, "Opening Balance");
    assert_eq!(first.cleared_status, "X");
  }
}
