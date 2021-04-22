mod date;
pub mod errors;
pub mod investment;
pub mod qif;
pub mod split;
pub mod transaction;

use investment::QifInvestment;
use split::QifSplit;
use transaction::QifTransaction;

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
) -> Result<qif::Qif<'a>, errors::QifParsingError> {
    let mut results: Vec<QifTransaction> = Vec::new();
    let mut investments: Vec<QifInvestment> = Vec::new();
    let mut result = qif::Qif {
        file_type: "",
        items: Vec::new(),
        investments: Vec::new(),
    };
    let mut current_item = QifTransaction::default();
    let mut current_investment = QifInvestment::default();
    let mut is_investment = false;
    let lines: Vec<&str> = qif_content.lines().collect();

    for line in lines {
        if line.starts_with("!Type:Invst") {
            is_investment = true;
        }
        if line.starts_with("!Type") {
            result.file_type = &line[6..];
        }
        if line.starts_with("^") {
            if is_investment {
                investments.push(current_investment);
            } else {
                results.push(current_item);
            }
            current_item = QifTransaction::default();
            current_investment = QifInvestment::default();
        }
        if is_investment {
            match parse_investment(line, &mut current_investment, date_format) {
                Err(err) => return Err(err),
                Ok(()) => (),
            }
        } else {
            match parse_line(line, &mut current_item, date_format) {
                Err(err) => return Err(err),
                Ok(()) => (),
            }
        }
    }

    result.items = results;
    result.investments = investments;

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

fn parse_investment<'a>(
    line: &'a str,
    item: &mut QifInvestment<'a>,
    date_format: &str,
) -> Result<(), errors::QifParsingError> {
    match &line[..1] {
        "T" | "U" => {
            item.amount = match parse_number(line) {
                Err(err) => return Err(err),
                Ok(amount) => amount,
            };
        }
        "D" => {
            item.date = match date::parse_date(&line[1..], date_format) {
                Err(err) => return Err(err),
                Ok(date) => date,
            };
        }
        "C" => item.cleared_status = &line[1..],
        "M" => item.memo = &line[1..],
        "N" => item.action = &line[1..],
        "Y" => item.security_name = &line[1..],
        "I" => {
            item.price = match parse_number(line) {
                Err(err) => return Err(err),
                Ok(amount) => amount,
            };
        }
        "Q" => {
            item.quantity = match parse_number(line) {
                Err(err) => return Err(err),
                Ok(amount) => amount,
            };
        }

        _ => {}
    };

    Ok(())
}

fn parse_line<'a>(
    line: &'a str,
    item: &mut QifTransaction<'a>,
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
        "M" => item.memo = &line[1..],
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

    #[test]
    fn parse_number_test() {
        let parsed = parse_number("X123.45").unwrap();
        assert_eq!(parsed, 123.45);

        let parsed = parse_number("X-123.45").unwrap();
        assert_eq!(parsed, -123.45);

        let parsed = parse_number("X+123.45").unwrap();
        assert_eq!(parsed, 123.45);

        let parsed = parse_number("X123").unwrap();
        assert_eq!(parsed, 123.0);
    }
}
