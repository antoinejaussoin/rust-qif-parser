use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents an Investment
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct QifInvestment<'a> {
    pub date: String,
    pub amount: f64,
    pub memo: &'a str,
    pub cleared_status: &'a str,
    /// Action from the `N` line: Buy, Sell, Div, StkSplit, and the other Quicken actions.
    pub action: &'a str,
    pub security_name: &'a str,
    pub price: f64,
    pub quantity: f64,
    /// Commission from the `O` line.
    pub commission_cost: f64,
    /// Cash moved between accounts, from the `$` line.
    pub amount_transferred: f64,
    /// Text from the `P` line (transfers and reminders).
    #[serde(default)]
    pub description: &'a str,
    /// Category, class, or transfer account from the `L` line.
    /// Transfer accounts are wrapped in brackets. `MiscIncX` and `MiscExpX` may contain a `|`.
    #[serde(default)]
    pub category: &'a str,
    /// Account this investment belongs to, from the preceding `!Account` record.
    #[serde(default)]
    pub account: &'a str,
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
