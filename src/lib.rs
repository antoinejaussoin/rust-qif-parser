mod date;
mod errors;

/// Represents a QIF file
/// It has a file_type (Bank, etc.) and a collection of items
pub struct Qif<'a> {
    /// File type can be one of: Cash, Bank, CCard, Invst, Oth A, Oth L, Invoice
    pub file_type: &'a str,
    pub items: Vec<QifItem<'a>>,
}

/// Represents a transaction
/// It has a date and an amount, and possibly some splits
pub struct QifItem<'a> {
    /// Parsed date, with format YYYY-MM-DD
    pub date: String,
    pub amount: f64,
    pub payee: &'a str,
    pub category: &'a str,
    pub cleared_status: &'a str,
    pub address: Vec<&'a str>,
    pub splits: Vec<QifSplit<'a>>,
    pub number_of_the_check: &'a str,
}

/// Represent a Split, which is basically a portion of a transaction
pub struct QifSplit<'a> {
    pub category: &'a str,
    pub memo: &'a str,
    pub amount: f64,
    pub number_of_the_check: &'a str,
}

fn empty_item<'a>() -> QifItem<'a> {
    QifItem {
        date: "".to_string(),
        amount: 0.0,
        payee: "",
        category: "",
        cleared_status: "",
        splits: Vec::new(),
        address: Vec::new(),
        number_of_the_check: "",
    }
}

/// This is the parsing function. It takes the text content of your QIF file as an argument,
/// and the date format.
///
/// Indeed, the date in a QIF file doesn't have a pre-determined format, which means you could
/// receive QIF files with completely different formats.
/// Please use, for the date_format, the format you would use with Chrono (https://docs.rs/chrono/0.4.13/chrono/format/strftime/index.html#specifiers)
///
/// Some examples: (all for November 1st, 1982)
/// 01/11/1982 -> %d/%m/%Y
/// 01/11/82   -> %d/%m/%y
/// 11/01/1982 -> %m/%d/%Y
/// 11/01'1982 -> %m/%d'%Y
///
/// The parser will then return a Qif data structure or an error
pub fn parse<'a>(
    qif_content: &'a str,
    date_format: &str,
) -> Result<Qif<'a>, errors::QifParsingError> {
    let mut results: Vec<QifItem> = Vec::new();
    let mut result = Qif {
        file_type: "",
        items: Vec::new(),
    };
    let mut current = empty_item();
    let lines: Vec<&str> = qif_content.lines().collect();

    for line in lines {
        if line.starts_with("!Type") {
            result.file_type = &line[6..];
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

fn parse_number(line: &str) -> Result<f64, errors::QifParsingError> {
    match line[1..].to_string().trim().replace(',', "").parse() {
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

fn parse_line<'a>(
    line: &'a str,
    item: &mut QifItem<'a>,
    date_format: &str,
) -> Result<(), errors::QifParsingError> {
    match &line[..1] {
        "T" | "U" => {
            item.amount = match parse_number(line) {
                Err(err) => return Err(err),
                Ok(amount) => amount,
            };
        }
        "P" => item.payee = &line[1..],
        "L" => item.category = &line[1..],
        "D" => {
            item.date = match date::parse_date(&line[1..], date_format) {
                Err(err) => return Err(err),
                Ok(date) => date,
            };
        }
        "C" => item.cleared_status = &line[1..],
        "A" => item.address.push(&line[1..]),
        "N" => {
            match item.splits.last_mut() {
                None => item.number_of_the_check = &line[1..],
                Some(item) => item.number_of_the_check = &line[1..],
            };
        }
        // Splits
        "S" => {
            let split = QifSplit {
                category: &line[1..],
                memo: "",
                amount: 0.0,
                number_of_the_check: "",
            };
            item.splits.push(split);
        }
        "E" => {
            let split = match item.splits.last_mut() {
                None => {
                    return Err(errors::QifParsingError::new(
                        "There should be a split item here",
                    ))
                }
                Some(item) => item,
            };
            split.memo = &line[1..];
        }
        "$" => {
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
        _ => {}
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wikipedia_simple_example() {
        let content = fs::read_to_string("data/wikipedia_simple.qif").unwrap();
        let result = parse(&content, "%m/%d/%y").unwrap();
        assert!(content.len() > 0);
        // QIF metadata
        assert_eq!(result.file_type, "Bank");

        // Items
        assert_eq!(result.items.len(), 3);

        // First items
        let first = &result.items[0];
        assert_eq!(first.date, "2010-03-03");
        assert_eq!(first.amount, -379.0);
        assert_eq!(first.category, "");
        assert_eq!(first.payee, "CITY OF SPRINGFIELD");
        assert_eq!(first.cleared_status, "");

        // Second items
        let second = &result.items[1];
        assert_eq!(second.date, "2010-03-04");
        assert_eq!(second.amount, -20.28);
        assert_eq!(second.category, "");
        assert_eq!(second.payee, "YOUR LOCAL SUPERMARKET");
        assert_eq!(second.cleared_status, "");
    }

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

        // First items
        let first = &result.items[0];
        assert_eq!(first.date, "2020-05-19");
        assert_eq!(first.amount, 500.0);
        assert_eq!(first.category, "");
        assert_eq!(first.payee, "REM CHQ REF1234");
        assert_eq!(first.cleared_status, "");

        // Third items
        let third = &result.items[2];
        assert_eq!(third.date, "2020-06-02");
        assert_eq!(third.amount, -9.59);
        assert_eq!(third.category, "");
        assert_eq!(third.payee, "KIMSUFI CARTE 1234 PAIEMENT CB 0106 ROUBAIX");
        assert_eq!(third.address.len(), 0);
    }

    #[test]
    fn test_nasty_example() {
        let content = fs::read_to_string("data/nasty.qif").unwrap();
        let result = parse(&content, "%d/%m/%Y").unwrap();
        assert!(content.len() > 0);

        // QIF metadata
        assert_eq!(result.file_type, "Bank");

        // Items
        assert_eq!(result.items.len(), 6);

        // First items
        let first = &result.items[0];
        assert_eq!(first.date, "2018-08-27");
        assert_eq!(first.amount, 10_000.0);
        assert_eq!(first.category, "");
        assert_eq!(first.payee, "Jane Doe");
        assert_eq!(first.cleared_status, "");

        // Second items
        let second = &result.items[1];
        assert_eq!(second.date, "2018-08-27");
        assert_eq!(second.amount, -10_000_000.0);
        assert_eq!(second.category, "Shopping");
        assert_eq!(second.payee, "Huge Amount 😅 with UTF8");
        assert_eq!(second.address.len(), 3);
        assert_eq!(second.address[0], "Address line 1");
        assert_eq!(second.address[1], "Address line 2");
        assert_eq!(second.address[2], "Address line 3");

        // Item with + sign on amount
        let third = &result.items[2];
        assert_eq!(third.amount, 123.0);

        // Item with missing leading 0 on date
        let fourth = &result.items[3];
        assert_eq!(fourth.date, "2018-08-28");

        // Item with cheque number
        let fifth = &result.items[4];
        assert_eq!(fifth.number_of_the_check, "CHQ100");

        // Item with cheque number on the splits
        let sixth = &result.items[5];
        assert_eq!(sixth.number_of_the_check, "");
        assert_eq!(sixth.splits[0].number_of_the_check, "CHQ101");
        assert_eq!(sixth.splits[1].number_of_the_check, "CHQ102");
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
