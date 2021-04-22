use super::investment::QifInvestment;
use super::transaction::QifTransaction;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a QIF file
/// It has a file_type (Bank, etc.) and a collection of items
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Qif<'a> {
    /// File type can be one of: Cash, Bank, CCard, Invst, Oth A, Oth L, Invoice
    pub file_type: &'a str,
    pub transactions: Vec<QifTransaction<'a>>,
    pub investments: Vec<QifInvestment<'a>>,
}

impl<'a> fmt::Display for Qif<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {} transactions, {} investments",
            self.file_type,
            self.transactions.len(),
            self.investments.len()
        )
    }
}
