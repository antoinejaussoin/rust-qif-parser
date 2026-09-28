use super::split::QifSplit;
use serde::{Deserialize, Serialize};
use std::fmt;

/// One invoice line item, from the `X` fields on a business transaction.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QifInvoiceLine<'a> {
    /// Description from `XS`, plus any following continuation lines.
    pub description: Vec<&'a str>,
    /// Item category from `XN`.
    pub category: &'a str,
    /// Quantity from `X#`.
    pub quantity: f64,
    /// Unit price from `X$`.
    pub unit_price: f64,
    /// Set when an `XF` line is present (`XFT`).
    pub taxable: bool,
    /// Class from `XK`.
    pub class_name: &'a str,
}

/// Quicken Home & Business fields stored on an invoice or bill transaction.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QifInvoice<'a> {
    /// `XI` value: `1` for an invoice, `3` for a payment.
    pub kind: &'a str,
    /// Due date from `XE`, normalized to YYYY-MM-DD.
    pub due_date: String,
    /// Number of payments from `XU`.
    pub payment_count: f64,
    /// Payment dates from `XD` lines, normalized to YYYY-MM-DD.
    pub payment_dates: Vec<String>,
    /// Payment amounts from `XY` lines.
    pub payment_amounts: Vec<f64>,
    /// Tax account from `XC`.
    pub tax_account: &'a str,
    /// Tax rate from `XR`.
    pub tax_rate: f64,
    /// Tax amount from `XT`.
    pub tax_amount: f64,
    /// Purchase order number from `XP`.
    pub po_number: &'a str,
    /// Customer message from `XM`.
    pub customer_message: &'a str,
    /// Ship-to address lines from `XA`.
    pub shipping_address: Vec<&'a str>,
    pub lines: Vec<QifInvoiceLine<'a>>,
}

/// Represents a transaction
/// It has a date and an amount, and possibly some splits
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct QifTransaction<'a> {
    /// Parsed date, with format YYYY-MM-DD
    pub date: String,
    pub amount: f64,
    pub memo: &'a str,
    pub payee: &'a str,
    pub category: &'a str,
    pub cleared_status: &'a str,
    pub address: Vec<&'a str>,
    pub splits: Vec<QifSplit<'a>>,
    pub number_of_the_check: &'a str,
    /// Account this transaction belongs to, from the preceding `!Account` record.
    #[serde(default)]
    pub account: &'a str,
    /// Set when an `F` line flags the transaction as a reimbursable business expense.
    #[serde(default)]
    pub reimbursable: bool,
    /// Present when the transaction carries Quicken business `X` fields.
    #[serde(default)]
    pub invoice: Option<QifInvoice<'a>>,
}

impl<'a> fmt::Display for QifTransaction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.date, self.amount, self.memo, self.payee
        )
    }
}
