use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a Split, which is basically a portion of a transaction
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QifSplit<'a> {
    pub category: &'a str,
    pub memo: &'a str,
    pub amount: f64,
    pub number_of_the_check: &'a str,
}

impl<'a> fmt::Display for QifSplit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.amount, self.memo, self.category)
    }
}
