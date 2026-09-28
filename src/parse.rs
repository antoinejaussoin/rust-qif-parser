use super::date::parse_date;
use super::errors::QifParsingError;
use super::investment::QifInvestment;
use super::number::{parse_amount, parse_price_value};
use super::qif::Qif;
use super::records::{
    QifAccount, QifCategory, QifClass, QifMemorized, QifPrice, QifSecurity, QifTag,
};
use super::split::QifSplit;
use super::transaction::{QifInvoice, QifInvoiceLine, QifTransaction};

/// Parse the text of a QIF file.
///
/// `date_format` is a [chrono strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers)
/// pattern, because QIF does not define one date format. Examples for 1 November 1982:
///
/// - `01/11/1982` -> `%d/%m/%Y`
/// - `01/11/82` -> `%d/%m/%y`
/// - `11/01/1982` -> `%m/%d/%Y`
/// - `11/01'1982` -> `%m/%d'%Y`
///
/// Quicken sometimes pads a single-digit day with a space (`6/ 1/94`). That form is accepted
/// for the usual `%d` / `%m` patterns. A space that belongs to the format itself, such as
/// the `' 7` year in `12/21' 7`, is left for `date_format` to match.
///
/// Bank, cash, card, asset, liability, invoice and bill sections become [`Qif::transactions`].
/// Investment sections become [`Qif::investments`]. Account, category, class, tag, security,
/// price and memorized sections are returned on their own fields. `!Account` / `TBank` is an
/// account type, not an amount, and a later `!Type:Bank` is parsed as transactions again
/// after an investment section.
///
/// A record that ends at the next header or at the end of the file is kept, even when the
/// closing `^` is missing. A `^` that does not follow any fields is ignored.
pub fn parse<'a>(qif_content: &'a str, date_format: &str) -> Result<Qif<'a>, QifParsingError> {
    let mut parser = Parser::new(date_format);
    parser.parse(qif_content)?;
    Ok(parser.result)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    /// Lines before the first header are ordinary transactions. Some exports omit `!Type`.
    None,
    Bank,
    Investment,
    Account,
    Category,
    Class,
    Tag,
    Memorized,
    Security,
    Prices,
    Budget,
    /// Lists this parser does not model (`Payee`, `Invitem`, `Template`, unknown types).
    Skip,
}

enum Directive<'a> {
    Section(Section, Option<&'a str>),
    Option(&'a str),
    Clear,
    Unknown,
}

struct Parser<'a, 'f> {
    date_format: &'f str,
    result: Qif<'a>,
    section: Section,
    current_account: &'a str,
    touched: bool,
    /// The previous line was `XS`. Following lines that do not start with `X` extend it.
    xs_open: bool,
    txn: QifTransaction<'a>,
    inv: QifInvestment<'a>,
    account: QifAccount<'a>,
    category: QifCategory<'a>,
    class_item: QifClass<'a>,
    tag: QifTag<'a>,
    security: QifSecurity<'a>,
    memorized_lines: Vec<&'a str>,
}

impl<'a, 'f> Parser<'a, 'f> {
    fn new(date_format: &'f str) -> Self {
        Self {
            date_format,
            result: Qif::default(),
            section: Section::None,
            current_account: "",
            touched: false,
            xs_open: false,
            txn: QifTransaction::default(),
            inv: QifInvestment::default(),
            account: QifAccount::default(),
            category: QifCategory::default(),
            class_item: QifClass::default(),
            tag: QifTag::default(),
            security: QifSecurity::default(),
            memorized_lines: Vec::new(),
        }
    }

    fn parse(&mut self, content: &'a str) -> Result<(), QifParsingError> {
        let content = content.strip_prefix('\u{feff}').unwrap_or(content);
        for line in content.split(['\n', '\r']) {
            let line = line.trim_start();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(directive) = directive(line) {
                self.apply_directive(directive)?;
                continue;
            }
            if line.trim() == "^" {
                self.finish_record()?;
                continue;
            }
            self.apply_line(line)?;
        }
        self.finish_record()?;
        Ok(())
    }

    fn apply_directive(&mut self, directive: Directive<'a>) -> Result<(), QifParsingError> {
        match directive {
            Directive::Section(section, file_type) => {
                self.finish_record()?;
                if let Some(file_type) = file_type {
                    self.result.file_type = file_type;
                }
                self.section = section;
            }
            Directive::Option(name) => self.result.options.push(name),
            Directive::Clear | Directive::Unknown => {}
        }
        Ok(())
    }

    fn apply_line(&mut self, line: &'a str) -> Result<(), QifParsingError> {
        match self.section {
            Section::None | Section::Bank => self.parse_bank_field(line),
            Section::Investment => self.parse_investment_field(line),
            Section::Account => self.parse_account_field(line),
            Section::Category | Section::Budget => self.parse_category_field(line),
            Section::Class => self.parse_named_field(line, false),
            Section::Tag => self.parse_named_field(line, true),
            Section::Security => self.parse_security_field(line),
            Section::Memorized => {
                self.memorized_lines.push(line);
                self.touched = true;
                Ok(())
            }
            Section::Prices => {
                self.result
                    .prices
                    .push(parse_price_line(line, self.date_format)?);
                Ok(())
            }
            Section::Skip => Ok(()),
        }
    }

    fn finish_record(&mut self) -> Result<(), QifParsingError> {
        if !self.touched {
            self.reset_partial();
            return Ok(());
        }
        match self.section {
            Section::None | Section::Bank => {
                self.txn.account = self.current_account;
                self.result.transactions.push(std::mem::take(&mut self.txn));
            }
            Section::Investment => {
                self.inv.account = self.current_account;
                self.result.investments.push(std::mem::take(&mut self.inv));
            }
            Section::Account => {
                self.current_account = self.account.name;
                self.result.accounts.push(std::mem::take(&mut self.account));
            }
            Section::Category | Section::Budget => {
                if !self.category.income && !self.category.expense {
                    self.category.expense = true;
                }
                self.result
                    .categories
                    .push(std::mem::take(&mut self.category));
            }
            Section::Class => {
                self.result
                    .classes
                    .push(std::mem::take(&mut self.class_item));
            }
            Section::Tag => {
                self.result.tags.push(std::mem::take(&mut self.tag));
            }
            Section::Security => {
                self.result
                    .securities
                    .push(std::mem::take(&mut self.security));
            }
            Section::Memorized => self.parse_memorized_buffer()?,
            Section::Prices | Section::Skip => {}
        }
        self.reset_partial();
        Ok(())
    }

    fn reset_partial(&mut self) {
        self.txn = QifTransaction::default();
        self.inv = QifInvestment::default();
        self.account = QifAccount::default();
        self.category = QifCategory::default();
        self.class_item = QifClass::default();
        self.tag = QifTag::default();
        self.security = QifSecurity::default();
        self.memorized_lines.clear();
        self.xs_open = false;
        self.touched = false;
    }

    fn parse_bank_field(&mut self, line: &'a str) -> Result<(), QifParsingError> {
        // A line after XS that does not itself start a business field extends the description.
        if self.xs_open && !line.starts_with('X') {
            self.append_xs(line);
            self.touched = true;
            return Ok(());
        }
        let Some((code, value)) = split_code(line) else {
            return Ok(());
        };
        match code {
            'D' => self.txn.date = parse_date(value, self.date_format)?,
            'T' | 'U' => self.txn.amount = parse_amount(value)?,
            'P' => self.txn.payee = value,
            'M' => self.txn.memo = value,
            'L' => self.txn.category = value,
            'C' => self.txn.cleared_status = value,
            'A' => self.txn.address.push(value),
            'F' => self.txn.reimbursable = true,
            'N' => {
                if let Some(split) = self.txn.splits.last_mut() {
                    split.number_of_the_check = value;
                } else {
                    self.txn.number_of_the_check = value;
                }
            }
            'S' => self.txn.splits.push(QifSplit {
                category: value,
                memo: "",
                amount: 0.0,
                percentage: 0.0,
                number_of_the_check: "",
            }),
            'E' => self.last_split()?.memo = value,
            '$' => self.last_split()?.amount = parse_amount(value)?,
            '%' => self.last_split()?.percentage = parse_amount(value)?,
            'X' => {
                if !self.parse_invoice(value)? {
                    return Ok(());
                }
            }
            _ => return Ok(()),
        }
        self.touched = true;
        Ok(())
    }

    fn last_split(&mut self) -> Result<&mut QifSplit<'a>, QifParsingError> {
        self.txn
            .splits
            .last_mut()
            .ok_or_else(|| QifParsingError::new("There should be a split item here"))
    }

    fn parse_invoice(&mut self, rest: &'a str) -> Result<bool, QifParsingError> {
        let Some((sub, value)) = split_code(rest) else {
            return Ok(false);
        };
        self.xs_open = false;
        match sub {
            'I' => self.invoice().kind = value,
            'E' => self.invoice().due_date = parse_date(value, self.date_format)?,
            'U' => self.invoice().payment_count = parse_amount(value)?,
            'D' => {
                let date = parse_date(value, self.date_format)?;
                self.invoice().payment_dates.push(date);
            }
            'Y' => {
                let amount = parse_amount(value)?;
                self.invoice().payment_amounts.push(amount);
            }
            'C' => self.invoice().tax_account = value,
            'R' => self.invoice().tax_rate = parse_amount(value)?,
            'T' => self.invoice().tax_amount = parse_amount(value)?,
            'P' => self.invoice().po_number = value,
            'A' => self.invoice().shipping_address.push(value),
            'M' => self.invoice().customer_message = value,
            'S' => {
                self.invoice().lines.push(QifInvoiceLine {
                    description: vec![value],
                    ..QifInvoiceLine::default()
                });
                self.xs_open = true;
            }
            'N' => self.invoice_line().category = value,
            '#' => self.invoice_line().quantity = parse_amount(value)?,
            '$' => self.invoice_line().unit_price = parse_amount(value)?,
            'F' => self.invoice_line().taxable = true,
            'K' => self.invoice_line().class_name = value,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn invoice(&mut self) -> &mut QifInvoice<'a> {
        self.txn.invoice.get_or_insert_with(QifInvoice::default)
    }

    fn invoice_line(&mut self) -> &mut QifInvoiceLine<'a> {
        let invoice = self.invoice();
        if invoice.lines.is_empty() {
            invoice.lines.push(QifInvoiceLine::default());
        }
        invoice.lines.last_mut().expect("a line was just inserted")
    }

    fn append_xs(&mut self, line: &'a str) {
        if let Some(line_item) = self
            .txn
            .invoice
            .as_mut()
            .and_then(|invoice| invoice.lines.last_mut())
        {
            line_item.description.push(line);
        }
    }

    fn parse_investment_field(&mut self, line: &'a str) -> Result<(), QifParsingError> {
        let Some((code, value)) = split_code(line) else {
            return Ok(());
        };
        match code {
            'D' => self.inv.date = parse_date(value, self.date_format)?,
            'T' | 'U' => self.inv.amount = parse_amount(value)?,
            'N' => self.inv.action = value,
            'Y' => self.inv.security_name = value,
            'I' => self.inv.price = parse_price_value(value)?,
            'Q' => self.inv.quantity = parse_amount(value)?,
            'O' => self.inv.commission_cost = parse_amount(value)?,
            '$' => self.inv.amount_transferred = parse_amount(value)?,
            'C' => self.inv.cleared_status = value,
            'M' => self.inv.memo = value,
            'P' => self.inv.description = value,
            'L' => self.inv.category = value,
            _ => return Ok(()),
        }
        self.touched = true;
        Ok(())
    }

    fn parse_account_field(&mut self, line: &'a str) -> Result<(), QifParsingError> {
        let Some((code, value)) = split_code(line) else {
            return Ok(());
        };
        match code {
            'N' => self.account.name = value,
            'T' => self.account.account_type = value,
            'D' => self.account.description = value,
            'L' => self.account.credit_limit = parse_amount(value)?,
            '/' => self.account.statement_balance_date = parse_date(value, self.date_format)?,
            '$' => self.account.statement_balance = parse_amount(value)?,
            _ => return Ok(()),
        }
        self.touched = true;
        Ok(())
    }

    fn parse_category_field(&mut self, line: &'a str) -> Result<(), QifParsingError> {
        let Some((code, value)) = split_code(line) else {
            return Ok(());
        };
        match code {
            'N' => self.category.name = value,
            'D' => self.category.description = value,
            'T' => self.category.tax_related = true,
            'I' => self.category.income = true,
            'E' => self.category.expense = true,
            'B' => self.category.budget_amounts.push(parse_amount(value)?),
            'R' => self.category.tax_schedule = value,
            _ => return Ok(()),
        }
        self.touched = true;
        Ok(())
    }

    fn parse_named_field(&mut self, line: &'a str, tag: bool) -> Result<(), QifParsingError> {
        let Some((code, value)) = split_code(line) else {
            return Ok(());
        };
        match code {
            'N' if tag => self.tag.name = value,
            'D' if tag => self.tag.description = value,
            'N' => self.class_item.name = value,
            'D' => self.class_item.description = value,
            _ => return Ok(()),
        }
        self.touched = true;
        Ok(())
    }

    fn parse_security_field(&mut self, line: &'a str) -> Result<(), QifParsingError> {
        let Some((code, value)) = split_code(line) else {
            return Ok(());
        };
        match code {
            'N' => self.security.name = value,
            'S' => self.security.symbol = value,
            'T' => self.security.security_type = value,
            'G' => self.security.goal = value,
            _ => return Ok(()),
        }
        self.touched = true;
        Ok(())
    }

    fn parse_memorized_buffer(&mut self) -> Result<(), QifParsingError> {
        let lines = std::mem::take(&mut self.memorized_lines);
        let mut kind = "";
        let mut first_payment_date = String::new();
        let mut loan_years = 0.0;
        let mut payments_made = 0.0;
        let mut periods_per_year = 0.0;
        let mut interest_rate = 0.0;
        let mut current_balance = 0.0;
        let mut original_amount = 0.0;
        let mut body = Vec::new();

        for line in lines {
            let Some((code, value)) = split_code(line) else {
                continue;
            };
            match code {
                'K' => kind = value,
                '1' => first_payment_date = parse_date(value, self.date_format)?,
                '2' => loan_years = parse_amount(value)?,
                '3' => payments_made = parse_amount(value)?,
                '4' => periods_per_year = parse_amount(value)?,
                '5' => interest_rate = parse_amount(value)?,
                '6' => current_balance = parse_amount(value)?,
                '7' => original_amount = parse_amount(value)?,
                _ => body.push(line),
            }
        }

        let account = self.current_account;
        let investment = if kind.starts_with('I') {
            for line in body {
                self.parse_investment_field(line)?;
            }
            self.inv.account = account;
            Some(std::mem::take(&mut self.inv))
        } else {
            for line in body {
                self.parse_bank_field(line)?;
            }
            self.txn.account = account;
            None
        };
        let transaction = if investment.is_some() {
            QifTransaction::default()
        } else {
            std::mem::take(&mut self.txn)
        };

        self.result.memorized.push(QifMemorized {
            kind,
            transaction,
            investment,
            first_payment_date,
            loan_years,
            payments_made,
            periods_per_year,
            interest_rate,
            current_balance,
            original_amount,
        });
        Ok(())
    }
}

fn directive(line: &str) -> Option<Directive<'_>> {
    let line = line.trim();
    if !line.starts_with('!') {
        return None;
    }
    if eq_ignore_ascii_case(line, "!account") {
        return Some(Directive::Section(Section::Account, None));
    }
    if strip_prefix_ignore_ascii_case(line, "!clear:").is_some() {
        return Some(Directive::Clear);
    }
    if let Some(rest) = strip_prefix_ignore_ascii_case(line, "!option:") {
        return Some(Directive::Option(rest.trim()));
    }
    if let Some(rest) = strip_prefix_ignore_ascii_case(line, "!type:") {
        let file_type = rest.trim();
        return Some(Directive::Section(
            section_for_type(file_type),
            Some(file_type),
        ));
    }
    Some(Directive::Unknown)
}

fn section_for_type(file_type: &str) -> Section {
    match file_type.to_ascii_lowercase().as_str() {
        "bank" | "cash" | "ccard" | "oth a" | "oth l" | "oth s" | "invoice" | "tax" | "bill" => {
            Section::Bank
        }
        "invst" | "port" => Section::Investment,
        "cat" => Section::Category,
        "class" => Section::Class,
        "tag" => Section::Tag,
        "memorized" => Section::Memorized,
        "security" => Section::Security,
        "prices" => Section::Prices,
        "budget" => Section::Budget,
        _ => Section::Skip,
    }
}

fn split_code(line: &str) -> Option<(char, &str)> {
    let mut chars = line.chars();
    let code = chars.next()?;
    Some((code, chars.as_str()))
}

fn parse_price_line<'a>(line: &'a str, date_format: &str) -> Result<QifPrice<'a>, QifParsingError> {
    let fields = split_csv(line);
    if fields.len() != 3 {
        return Err(QifParsingError::new(&format!(
            "Could not parse security price line: '{line}'"
        )));
    }
    let symbol = unquote(fields[0]);
    let price = parse_price_value(unquote(fields[1])).map_err(|_| {
        QifParsingError::new(&format!("Could not parse security price line: '{line}'"))
    })?;
    let date = parse_date(unquote(fields[2]), date_format).map_err(|_| {
        QifParsingError::new(&format!("Could not parse security price line: '{line}'"))
    })?;
    Ok(QifPrice {
        symbol,
        price,
        date,
    })
}

fn split_csv(line: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    for (index, byte) in line.bytes().enumerate() {
        match byte {
            b'"' => in_quotes = !in_quotes,
            b',' if !in_quotes => {
                fields.push(&line[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    fields.push(&line[start..]);
    fields
}

fn unquote(field: &str) -> &str {
    let field = field.trim();
    if field.len() >= 2 && field.starts_with('"') && field.ends_with('"') {
        &field[1..field.len() - 1]
    } else {
        field
    }
}

fn eq_ignore_ascii_case(left: &str, right: &str) -> bool {
    left.len() == right.len() && left.as_bytes().eq_ignore_ascii_case(right.as_bytes())
}

fn strip_prefix_ignore_ascii_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    if value.len() >= prefix.len()
        && value.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
    {
        Some(&value[prefix.len()..])
    } else {
        None
    }
}
