use super::errors::QifParsingError;

/// Parse a QIF numeric field.
///
/// Quicken and bank exports do not agree on a single spelling of amounts, so this accepts:
/// - an optional leading `+` or `-`, or parentheses for a negative amount
/// - `$`, `£`, `€`, `¥` and spaces (including non-breaking spaces) as decoration
/// - `,` as a thousands separator (`1,234.56`, `30,020.00`) or, when it is the only
///   separator and is followed by 1 or 2 digits, as a decimal comma (`1,50`)
/// - `.` as a thousands separator when a comma is the decimal mark (`1.234,56`)
/// - `'` as a thousands separator (`10'000.00`, used by older Quicken versions)
/// - a leading `=` (`=746.36`). The Intuit sample copied by the Qif test suite writes
///   the second split this way; the number is taken as written
pub fn parse_amount(raw: &str) -> Result<f64, QifParsingError> {
    let original = raw;
    let mut body = raw.trim();
    if body.is_empty() {
        return amount_error(original);
    }
    if let Some(rest) = body.strip_prefix('=') {
        body = rest.trim();
    }

    let mut negative = false;
    if body.starts_with('(') && body.ends_with(')') && body.len() > 2 {
        negative = true;
        body = body[1..body.len() - 1].trim();
    }

    let mut cleaned = String::with_capacity(body.len());
    for c in body.chars() {
        match c {
            '$' | '£' | '€' | '¥' | '¤' | ' ' | '\u{00a0}' | '\u{202f}' | '\'' | '\u{2019}'
            | '_' => {}
            '−' => cleaned.push('-'),
            _ => cleaned.push(c),
        }
    }

    let mut num = cleaned.as_str().trim();
    if let Some(rest) = num.strip_prefix('+') {
        num = rest.trim();
    } else if let Some(rest) = num.strip_prefix('-') {
        negative = true;
        num = rest.trim();
    }
    if num.is_empty() {
        return amount_error(original);
    }

    let normalized = normalize_separators(num);
    match normalized.parse::<f64>() {
        Ok(value) if value.is_finite() => Ok(if negative { -value } else { value }),
        _ => amount_error(original),
    }
}

/// A share price, either a plain amount or a whole number plus a fraction (`50 1/2`).
pub fn parse_price_value(raw: &str) -> Result<f64, QifParsingError> {
    let raw = raw.trim();
    if let Some(price) = parse_fractional_price(raw) {
        return Ok(price);
    }
    parse_amount(raw)
}

fn parse_fractional_price(raw: &str) -> Option<f64> {
    let (whole, frac) = if let Some((whole, frac)) = raw.rsplit_once(|c: char| c.is_whitespace()) {
        if !frac.contains('/') {
            return None;
        }
        (whole.trim(), frac.trim())
    } else if raw.contains('/') {
        ("0", raw)
    } else {
        return None;
    };

    let (numerator, denominator) = frac.split_once('/')?;
    let whole: f64 = if whole.is_empty() {
        0.0
    } else {
        whole.parse().ok()?
    };
    let numerator: f64 = numerator.trim().parse().ok()?;
    let denominator: f64 = denominator.trim().parse().ok()?;
    if denominator == 0.0 {
        return None;
    }
    Some(whole + numerator / denominator)
}

/// Groups of three digits after a comma are thousands (`1,234` -> 1234), matching
/// Quicken and this crate's existing files. `1,000` is therefore one thousand, not a
/// 3-decimal European amount. A single comma followed by one or two digits is a
/// decimal comma (`1,5`, `1,50`).
fn normalize_separators(num: &str) -> String {
    let last_dot = num.rfind('.');
    let last_comma = num.rfind(',');
    match (last_dot, last_comma) {
        (Some(dot), Some(comma)) if comma > dot => {
            let mut normalized = String::with_capacity(num.len());
            let mut decimal_used = false;
            for (index, c) in num.chars().enumerate() {
                if c == '.' {
                    continue;
                }
                if c == ',' {
                    if index == comma && !decimal_used {
                        normalized.push('.');
                        decimal_used = true;
                    }
                    continue;
                }
                normalized.push(c);
            }
            normalized
        }
        (Some(_), Some(_)) => num.replace(',', ""),
        (None, Some(_)) if is_decimal_comma(num) => num.replace(',', "."),
        (None, Some(_)) => num.replace(',', ""),
        _ => num.to_string(),
    }
}

fn is_decimal_comma(num: &str) -> bool {
    let Some((whole, fraction)) = num.split_once(',') else {
        return false;
    };
    if num.matches(',').count() != 1 {
        return false;
    }
    (1..=2).contains(&fraction.len())
        && !whole.is_empty()
        && whole.chars().all(|c| c.is_ascii_digit())
        && fraction.chars().all(|c| c.is_ascii_digit())
}

fn amount_error(raw: &str) -> Result<f64, QifParsingError> {
    Err(QifParsingError::new(&format!(
        "Could not parse the following as a number: '{raw}'"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_and_signed_numbers() {
        assert_eq!(parse_amount("123.45").unwrap(), 123.45);
        assert_eq!(parse_amount("-123.45").unwrap(), -123.45);
        assert_eq!(parse_amount("+123.45").unwrap(), 123.45);
        assert_eq!(parse_amount("123").unwrap(), 123.0);
        assert_eq!(parse_amount("  -10.00  ").unwrap(), -10.0);
        assert_eq!(parse_amount("+123").unwrap(), 123.0);
        assert_eq!(parse_amount("0").unwrap(), 0.0);
        assert_eq!(parse_amount("-1000.0").unwrap(), -1000.0);
        assert_eq!(parse_amount(".5").unwrap(), 0.5);
        assert_eq!(parse_amount("5.").unwrap(), 5.0);
    }

    #[test]
    fn parses_us_thousands_separators() {
        assert_eq!(parse_amount("10,000.00").unwrap(), 10_000.0);
        assert_eq!(parse_amount("-10,000,000.00").unwrap(), -10_000_000.0);
        assert_eq!(parse_amount("-1,010.00").unwrap(), -1_010.0);
        assert_eq!(parse_amount("-30,020.00").unwrap(), -30_020.0);
        assert_eq!(parse_amount("1,000").unwrap(), 1_000.0);
        assert_eq!(parse_amount("1,234.56").unwrap(), 1_234.56);
        assert_eq!(parse_amount("4,896.201").unwrap(), 4_896.201);
    }

    #[test]
    fn parses_european_amounts() {
        assert_eq!(parse_amount("10.000,00").unwrap(), 10_000.0);
        assert_eq!(parse_amount("-1.234,56").unwrap(), -1_234.56);
        assert_eq!(parse_amount("1,50").unwrap(), 1.5);
        assert_eq!(parse_amount("1,5").unwrap(), 1.5);
        assert_eq!(parse_amount("12,50").unwrap(), 12.5);
        assert_eq!(parse_amount("1 234,56").unwrap(), 1_234.56);
        assert_eq!(parse_amount("10 000 000.00").unwrap(), 10_000_000.0);
    }

    #[test]
    fn parses_apostrophe_thousands_from_older_quicken() {
        assert_eq!(parse_amount("10'000.00").unwrap(), 10_000.0);
        assert_eq!(parse_amount("10'000,00").unwrap(), 10_000.0);
        assert_eq!(parse_amount("-1'234.50").unwrap(), -1_234.5);
    }

    #[test]
    fn parses_currency_symbols_parentheses_and_equals() {
        assert_eq!(parse_amount("$123.45").unwrap(), 123.45);
        assert_eq!(parse_amount("-$50.00").unwrap(), -50.0);
        assert_eq!(parse_amount("$-1,234.50").unwrap(), -1_234.5);
        assert_eq!(parse_amount("£1,000.00").unwrap(), 1_000.0);
        assert_eq!(parse_amount("€1.234,56").unwrap(), 1_234.56);
        assert_eq!(parse_amount("(123.45)").unwrap(), -123.45);
        assert_eq!(parse_amount("($1,234.56)").unwrap(), -1_234.56);
        assert_eq!(parse_amount("(1.234,56)").unwrap(), -1_234.56);
        assert_eq!(parse_amount("=746.36").unwrap(), 746.36);
        assert_eq!(parse_amount("=$1,000.00").unwrap(), 1_000.0);
        assert_eq!(parse_amount("−12.5").unwrap(), -12.5);
    }

    #[test]
    fn rejects_amounts_that_are_not_numbers() {
        for raw in [
            "", "   ", "-", "+", "Invst", "Bank", "abc", "1.2.3", "inf", "NaN",
        ] {
            let err = parse_amount(raw).unwrap_err();
            assert!(
                err.details.contains(raw.trim()),
                "error for {raw:?} was {}",
                err.details
            );
        }
    }

    #[test]
    fn parses_fractional_prices() {
        assert_eq!(parse_price_value("50 1/2").unwrap(), 50.5);
        assert_eq!(parse_price_value("10 3/8").unwrap(), 10.375);
        assert_eq!(parse_price_value("1/2").unwrap(), 0.5);
        assert_eq!(parse_price_value("50.50").unwrap(), 50.5);
        assert_eq!(parse_price_value("11.260").unwrap(), 11.26);
        assert_eq!(parse_price_value("  50   1/2 ").unwrap(), 50.5);
    }

    #[test]
    fn rejects_a_zero_denominator_fraction() {
        assert!(parse_price_value("50 1/0").is_err());
    }
}
