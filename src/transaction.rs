use super::split::QifSplit;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a transaction
/// It has a date and an amount, and possibly some splits
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
