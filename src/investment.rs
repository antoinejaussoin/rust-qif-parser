use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents an Investment
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QifInvestment<'a> {
    pub date: String,
    pub amount: f64,
    pub memo: &'a str,
    pub cleared_status: &'a str,
    pub action: &'a str,
    pub security_name: &'a str,
    pub price: f64,
    pub quantity: f64,
    pub commission_cost: f64,
    pub amount_transferred: f64,
}

impl<'a> fmt::Display for QifInvestment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.date, self.amount, self.action, self.security_name, self.memo
        )
    }
}
