use super::investment::QifInvestment;
use super::transaction::QifTransaction;
use serde::{Deserialize, Serialize};
use std::fmt;

/// An `!Account` record. Quicken writes these both as an account list and as the
/// header that names the account for the transactions that follow.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QifAccount<'a> {
    pub name: &'a str,
    pub account_type: &'a str,
    pub description: &'a str,
    pub credit_limit: f64,
    /// Statement date from the `/` line, normalized to YYYY-MM-DD. Empty when absent.
    pub statement_balance_date: String,
    pub statement_balance: f64,
}

impl fmt::Display for QifAccount<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.account_type)
    }
}

/// A category from `!Type:Cat` or a budget row from `!Type:Budget`.
///
/// When a category line does not say whether it is income or expense, Quicken
/// treats it as an expense. The parser does the same.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QifCategory<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub income: bool,
    pub expense: bool,
    /// Set when a `T` line is present. In a category list `T` is a flag, not an amount.
    pub tax_related: bool,
    pub tax_schedule: &'a str,
    /// One entry per `B` line. Budget files repeat `B` once per month.
    pub budget_amounts: Vec<f64>,
}

impl fmt::Display for QifCategory<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A class from `!Type:Class`.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QifClass<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

impl fmt::Display for QifClass<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A tag from `!Type:Tag`.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QifTag<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

impl fmt::Display for QifTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A security from `!Type:Security`.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QifSecurity<'a> {
    pub name: &'a str,
    pub symbol: &'a str,
    pub security_type: &'a str,
    pub goal: &'a str,
}

impl fmt::Display for QifSecurity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.symbol)
    }
}

/// One price from `!Type:Prices`.
///
/// Price lines are not letter-coded. They look like `"INTU",50 1/2,"6/30/98"`.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QifPrice<'a> {
    pub symbol: &'a str,
    pub price: f64,
    /// Normalized to YYYY-MM-DD using the same date format as the rest of the file.
    pub date: String,
}

impl fmt::Display for QifPrice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.symbol, self.price, self.date)
    }
}

/// A memorized transaction from `!Type:Memorized`.
///
/// `kind` is the letter from the `K` line: `C` check, `D` deposit, `P` payment,
/// `I` investment, `E` electronic payee. Investment memorized items (`KI`) fill
/// `investment`; every other kind fills `transaction`.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct QifMemorized<'a> {
    pub kind: &'a str,
    pub transaction: QifTransaction<'a>,
    pub investment: Option<QifInvestment<'a>>,
    pub first_payment_date: String,
    pub loan_years: f64,
    pub payments_made: f64,
    pub periods_per_year: f64,
    pub interest_rate: f64,
    pub current_balance: f64,
    pub original_amount: f64,
}

impl fmt::Display for QifMemorized<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(investment) = &self.investment {
            write!(f, "{investment}")
        } else {
            write!(f, "{}", self.transaction)
        }
    }
}
