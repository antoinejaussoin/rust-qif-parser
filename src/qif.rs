use super::investment::QifInvestment;
use super::records::{
    QifAccount, QifCategory, QifClass, QifMemorized, QifPrice, QifSecurity, QifTag,
};
use super::transaction::QifTransaction;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a QIF file.
///
/// `file_type` is the suffix of the last `!Type:` header in the file (`Bank`, `Invst`,
/// `Cat`, ...). A Quicken export contains several lists; each list is returned in its
/// own collection rather than being mixed into `transactions`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Qif<'a> {
    /// Suffix of the last `!Type:` header. Empty when the file has none.
    pub file_type: &'a str,
    pub transactions: Vec<QifTransaction<'a>>,
    pub investments: Vec<QifInvestment<'a>>,
    /// Names from `!Option:` lines, such as `AutoSwitch` and `AllXfr`.
    #[serde(default)]
    pub options: Vec<&'a str>,
    #[serde(default)]
    pub accounts: Vec<QifAccount<'a>>,
    #[serde(default)]
    pub categories: Vec<QifCategory<'a>>,
    #[serde(default)]
    pub classes: Vec<QifClass<'a>>,
    #[serde(default)]
    pub tags: Vec<QifTag<'a>>,
    #[serde(default)]
    pub securities: Vec<QifSecurity<'a>>,
    #[serde(default)]
    pub prices: Vec<QifPrice<'a>>,
    #[serde(default)]
    pub memorized: Vec<QifMemorized<'a>>,
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
