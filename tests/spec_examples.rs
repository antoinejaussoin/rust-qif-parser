//! Samples taken from published QIF descriptions, not invented for this crate.
//!
//! - GnuCash `file-format.txt` investment sample:
//!   https://github.com/Gnucash/gnucash/blob/stable/gnucash/import-export/qif-imp/file-format.txt
//! - Intuit's QIF examples, as vendored by the Qif Ruby gem:
//!   https://github.com/jemmyw/Qif/tree/master/spec/fixtures
//! - Wikipedia's Quicken import example (`!Account` before `!Type:Invst`):
//!   https://en.wikipedia.org/wiki/Quicken_Interchange_Format
//! - Security price lines from the same GnuCash / Intuit QIF99 description
//!   (`"INTU",50 1/2,"6/30/98"`).

use qif_parser::parse;
use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn parse_both<'a>(content: &'a str, date_format: &str) -> qif_parser::qif::Qif<'a> {
    let explicit = parse(content, date_format).unwrap();
    let automatic = parse(content, None).unwrap();
    assert_eq!(explicit, automatic);
    explicit
}

#[test]
fn gnucash_mutual_fund_sample() {
    let content = read("data/gnucash_investments.qif");
    let result = parse_both(&content, "%m/%d/%Y");

    assert_eq!(result.file_type, "Invst");
    assert!(result.transactions.is_empty());
    assert_eq!(result.accounts.len(), 1);
    assert_eq!(result.accounts[0].name, "Assets:Investments:Mutual Fund");
    assert_eq!(result.accounts[0].account_type, "Invst");

    assert_eq!(result.investments.len(), 2);
    let buy = &result.investments[0];
    assert_eq!(buy.account, "Assets:Investments:Mutual Fund");
    assert_eq!(buy.date, "2006-10-30");
    assert_eq!(buy.quantity, 0.9);
    assert_eq!(buy.amount, 500.0);
    assert_eq!(buy.description, "Purchase");
    assert_eq!(buy.action, "BuyX");
    assert_eq!(buy.category, "[Assets:Investments:Mutual Fund:Cash]");
    assert_eq!(buy.security_name, "FOO");

    let sell = &result.investments[1];
    assert_eq!(sell.date, "2006-11-28");
    assert_eq!(sell.quantity, 0.897);
    assert_eq!(sell.amount, 100.0);
    assert_eq!(sell.description, "Sale");
    assert_eq!(sell.action, "SellX");
    assert_eq!(sell.security_name, "FOO");
    assert_eq!(sell.account, "Assets:Investments:Mutual Fund");
}

#[test]
fn intuit_investment_sample_from_the_qif_gem() {
    let content = read("data/external/quicken_investment_account.qif");
    let result = parse_both(&content, "%m/%d/%y");

    assert_eq!(result.file_type, "Invst");
    assert!(result.transactions.is_empty());
    assert_eq!(result.investments.len(), 2);

    let opening = &result.investments[0];
    assert_eq!(opening.date, "1993-08-25");
    assert_eq!(opening.action, "ShrsIn");
    assert_eq!(opening.security_name, "ibm4");
    assert_eq!(opening.price, 11.26);
    assert_eq!(opening.quantity, 88.81);
    assert_eq!(opening.cleared_status, "X");
    assert_eq!(opening.amount, 1_000.0);
    assert_eq!(opening.memo, "Opening");
    assert_eq!(opening.commission_cost, 0.0);
    assert_eq!(opening.amount_transferred, 0.0);

    let buy = &result.investments[1];
    assert_eq!(buy.date, "1993-08-25");
    assert_eq!(buy.action, "BuyX");
    assert_eq!(buy.security_name, "ibm4");
    assert_eq!(buy.price, 11.03);
    assert_eq!(buy.quantity, 9.066);
    assert_eq!(buy.amount, 100.0);
    assert_eq!(buy.memo, "Est. price as of 8/25/93");
    assert_eq!(buy.category, "[CHECKING]");
    assert_eq!(buy.amount_transferred, 100.0);
}

#[test]
fn intuit_bank_sample_from_the_qif_gem() {
    // D6/1/94 is ambiguous. The Intuit commentary around this sample is a US date,
    // so the caller passes month/day. Day/month would also parse, as 6 January.
    let content = read("data/external/quicken_non_investment_account.qif");
    let result = parse_both(&content, "%m/%d/%y");

    assert_eq!(result.file_type, "Bank");
    assert_eq!(result.transactions.len(), 3);

    let mortgage = &result.transactions[0];
    assert_eq!(mortgage.date, "1994-06-01");
    assert_eq!(mortgage.amount, -1_000.0);
    assert_eq!(mortgage.cleared_status, "X");
    assert_eq!(mortgage.number_of_the_check, "1005");
    assert_eq!(mortgage.payee, "Bank Of Mortgage");
    assert_eq!(mortgage.memo, "Memo");
    assert_eq!(mortgage.category, "[linda]");
    assert_eq!(mortgage.splits.len(), 2);
    assert_eq!(mortgage.splits[0].category, "[linda]");
    assert_eq!(mortgage.splits[0].memo, "Cash");
    assert_eq!(mortgage.splits[0].amount, -253.64);
    // The sample writes `$=746.36`. The `=` is not a minus sign; the Qif gem reads 746.36.
    assert_eq!(mortgage.splits[1].category, "Mort Int");
    assert_eq!(mortgage.splits[1].amount, 746.36);

    let deposit = &result.transactions[1];
    assert_eq!(deposit.date, "1994-06-02");
    assert_eq!(deposit.amount, 75.0);
    assert_eq!(deposit.payee, "Deposit");
    assert!(deposit.splits.is_empty());

    let check = &result.transactions[2];
    assert_eq!(check.date, "1994-06-03");
    assert_eq!(check.amount, -10.0);
    assert_eq!(check.payee, "Anthony Hopkins");
    assert_eq!(check.memo, "Film");
    assert_eq!(check.category, "Entertain");
    assert_eq!(
        check.address,
        vec!["P.O. Box 27027", "Tucson, AZ", "85726", "", "", ""]
    );
}

#[test]
fn qif_gem_splits_include_a_split_written_before_the_amount() {
    let content = read("data/external/splits.qif");
    let result = parse_both(&content, "%m/%d/%y");
    assert_eq!(result.transactions.len(), 2);

    let first = &result.transactions[0];
    assert_eq!(first.splits.len(), 2);
    assert_eq!(first.splits[0].category, "[steve]");
    assert_eq!(first.splits[0].memo, "Cash");
    assert_eq!(first.splits[0].amount, -253.64);
    assert_eq!(first.splits[1].category, "Mort Int");
    assert_eq!(first.splits[1].amount, 746.36);
    assert_eq!(first.category, "[linda]");

    let second = &result.transactions[1];
    assert_eq!(second.amount, 75.0);
    assert_eq!(second.payee, "Deposit");
    assert_eq!(second.splits.len(), 1);
    assert_eq!(second.splits[0].category, "category 1");
    assert_eq!(second.splits[0].amount, 23.0);
}

#[test]
fn qif_gem_space_padded_dates_and_unpadded_days() {
    let spaced_file = read("data/external/3_records_spaced.qif");
    let spaced = parse_both(&spaced_file, "%m/%d/%y");
    assert_eq!(spaced.transactions.len(), 3);
    assert_eq!(spaced.transactions[0].date, "2010-01-01");
    assert_eq!(spaced.transactions[0].amount, -10.0);
    assert_eq!(spaced.transactions[0].category, "Debit");
    assert_eq!(spaced.transactions[1].date, "2010-06-01");
    assert_eq!(spaced.transactions[1].amount, -20.0);
    assert_eq!(spaced.transactions[2].date, "2010-12-29");
    assert_eq!(spaced.transactions[2].amount, 30.0);
    assert_eq!(spaced.transactions[2].category, "Credit");
    assert_eq!(spaced.transactions[2].payee, "Description");
    assert_eq!(spaced.transactions[2].memo, "Reference");

    let unpadded_file = read("data/external/3_records_dmyy.qif");
    let unpadded = parse_both(&unpadded_file, "%d/%m/%y");
    assert_eq!(unpadded.transactions[0].date, "2010-01-01");
    assert_eq!(unpadded.transactions[1].date, "2010-06-01");
    assert_eq!(unpadded.transactions[2].date, "2010-12-29");
    assert_eq!(unpadded.transactions[2].amount, 30.0);
}

#[test]
fn qif_gem_amounts_with_thousands_commas() {
    let content = read("data/external/3_records_separator.qif");
    let result = parse_both(&content, "%d/%m/%Y");
    let amounts: Vec<f64> = result.transactions.iter().map(|txn| txn.amount).collect();
    assert_eq!(amounts, vec![-1_010.0, -30_020.0, 30.0]);
    assert_eq!(result.transactions[0].date, "2010-01-01");
    assert_eq!(result.transactions[2].date, "2010-12-29");
}

#[test]
fn a_caret_immediately_after_the_header_is_not_a_transaction() {
    let content = read("data/external/3_records_invalid_header.qif");
    let result = parse_both(&content, "%d/%m/%y");
    assert_eq!(result.transactions.len(), 3);
    assert_eq!(result.transactions[0].date, "2010-01-01");
    assert_eq!(result.transactions[0].amount, -10.0);
    assert_eq!(result.transactions[0].payee, "Description");
    assert_eq!(result.transactions[2].date, "2010-12-29");
}

#[test]
fn wikipedia_account_header_before_an_investment_register() {
    // This is the example Wikipedia gives for importing into an existing brokerage account.
    // `TInvst` is the account type. It must not be read as a transaction amount.
    let content = "\
!Account
NJoint Brokerage Account
TInvst
^
!Type:Invst
D12/21' 7
NBuy
YIBM
T11010.00
I110.10
Q100
MPurchase of 100 shares of IBM stock on 21 December 2007 at $110.10 per share
^
";
    let result = parse_both(content, "%m/%d'%y");
    assert_eq!(result.file_type, "Invst");
    assert!(result.transactions.is_empty());
    assert_eq!(result.accounts.len(), 1);
    assert_eq!(result.accounts[0].name, "Joint Brokerage Account");
    assert_eq!(result.accounts[0].account_type, "Invst");
    assert_eq!(result.investments.len(), 1);
    assert_eq!(result.investments[0].account, "Joint Brokerage Account");
    assert_eq!(result.investments[0].date, "2007-12-21");
    assert_eq!(result.investments[0].action, "Buy");
    assert_eq!(result.investments[0].security_name, "IBM");
    assert_eq!(result.investments[0].amount, 11_010.0);
    assert_eq!(result.investments[0].price, 110.10);
    assert_eq!(result.investments[0].quantity, 100.0);
}

#[test]
fn security_price_lines_from_the_qif_specification() {
    let content = read("data/prices.qif");
    let result = parse_both(&content, "%m/%d/%y");
    assert_eq!(result.file_type, "Prices");
    assert!(result.transactions.is_empty());
    assert!(result.investments.is_empty());
    assert_eq!(result.prices.len(), 5);

    assert_eq!(result.prices[0].symbol, "INTU");
    assert_eq!(result.prices[0].price, 50.5);
    assert_eq!(result.prices[0].date, "1998-06-30");

    assert_eq!(result.prices[1].symbol, "INTU");
    assert_eq!(result.prices[1].price, 50.5);
    assert_eq!(result.prices[1].date, "1998-06-30");

    assert_eq!(result.prices[2].symbol, "BRK.B");
    assert_eq!(result.prices[2].price, 212.25);
    assert_eq!(result.prices[2].date, "2011-03-17");

    assert_eq!(result.prices[3].symbol, "FOO, BAR");
    assert_eq!(result.prices[3].price, 1.25);
    assert_eq!(result.prices[3].date, "2020-01-02");

    assert_eq!(result.prices[4].symbol, "INTC");
    assert_eq!(result.prices[4].price, 10.375);
    assert_eq!(result.prices[4].date, "2011-03-17");
}

#[test]
fn category_class_tag_security_and_budget_lists() {
    let content = read("data/lists.qif");
    let result = parse_both(&content, "%m/%d/%Y");
    assert_eq!(result.file_type, "Budget");
    assert!(result.transactions.is_empty());

    assert_eq!(result.categories.len(), 5);
    let food = &result.categories[0];
    assert_eq!(food.name, "Food:Groceries");
    assert_eq!(food.description, "Everyday food");
    assert!(food.expense);
    assert!(!food.income);
    assert!(food.tax_related);
    assert_eq!(food.tax_schedule, "Schedule A");
    assert_eq!(food.budget_amounts, vec![250.0, 300.5]);

    let salary = &result.categories[1];
    assert_eq!(salary.name, "Salary");
    assert_eq!(salary.description, "Paycheck");
    assert!(salary.income);
    assert!(!salary.expense);
    assert!(!salary.tax_related);

    let misc = &result.categories[2];
    assert_eq!(misc.name, "Misc");
    assert!(misc.expense);
    assert!(!misc.income);

    let both = &result.categories[3];
    assert!(both.income);
    assert!(both.expense);

    // A budget list is not folded into the earlier category of the same name.
    let budget = &result.categories[4];
    assert_eq!(budget.name, "Food:Groceries");
    assert_eq!(budget.budget_amounts, vec![10.0, 20.0]);
    assert!(budget.expense);

    assert_eq!(result.classes.len(), 2);
    assert_eq!(result.classes[0].name, "Tax");
    assert_eq!(result.classes[0].description, "Tax deductible");
    assert_eq!(result.classes[1].name, "Vacation");
    assert_eq!(result.classes[1].description, "");

    assert_eq!(result.tags.len(), 1);
    assert_eq!(result.tags[0].name, "Holiday");
    assert_eq!(result.tags[0].description, "Time off");

    assert_eq!(result.securities.len(), 2);
    assert_eq!(result.securities[0].name, "Hexcel Corp");
    assert_eq!(result.securities[0].symbol, "HXL");
    assert_eq!(result.securities[0].security_type, "Stock");
    assert_eq!(result.securities[0].goal, "Growth");
    assert_eq!(result.securities[1].name, "Boston Pptys");
    assert_eq!(result.securities[1].symbol, "BXP");
    assert_eq!(result.securities[1].security_type, "Mutual Fund");
    assert_eq!(result.securities[1].goal, "");
}

#[test]
fn quicken_autoswitch_export_keeps_each_register_with_its_account() {
    let content = read("data/autoswitch.qif");
    let result = parse_both(&content, "%m/%d/%Y");

    assert_eq!(result.options, vec!["AutoSwitch", "AllXfr"]);
    assert_eq!(result.file_type, "Bank");
    assert_eq!(result.transactions.len(), 2);
    assert_eq!(result.investments.len(), 2);

    assert_eq!(result.accounts[0].name, "Checking");
    assert_eq!(result.accounts[0].account_type, "Bank");
    assert_eq!(result.accounts[0].description, "Primary checking");
    assert_eq!(result.accounts[2].name, "Visa");
    assert_eq!(result.accounts[2].account_type, "CCard");
    assert_eq!(result.accounts[2].description, "Rewards card");
    assert_eq!(result.accounts[2].credit_limit, 2_500.0);
    assert_eq!(result.accounts[2].statement_balance_date, "2020-01-15");
    assert_eq!(result.accounts[2].statement_balance, -40.5);

    let shop = &result.transactions[0];
    assert_eq!(shop.account, "Checking");
    assert_eq!(shop.date, "2020-01-02");
    assert_eq!(shop.amount, -1_234.5);
    assert_eq!(shop.payee, "Grocery Store");
    assert_eq!(shop.category, "Groceries/Personal");
    assert_eq!(shop.number_of_the_check, "1024");
    assert_eq!(shop.cleared_status, "*");
    assert_eq!(shop.memo, "weekly shop");
    assert_eq!(shop.address, vec!["1 High Street", "London"]);
    assert!(shop.reimbursable);
    assert_eq!(shop.splits.len(), 2);
    assert_eq!(shop.splits[0].category, "Groceries");
    assert_eq!(shop.splits[0].memo, "food");
    assert_eq!(shop.splits[0].amount, -40.0);
    assert_eq!(shop.splits[0].percentage, 75.0);
    assert_eq!(shop.splits[1].category, "Household");
    assert_eq!(shop.splits[1].amount, -10.0);
    assert_eq!(shop.splits[1].percentage, 25.0);

    let buy = &result.investments[0];
    assert_eq!(buy.account, "Brokerage");
    assert_eq!(buy.date, "2020-03-04");
    assert_eq!(buy.action, "Buy");
    assert_eq!(buy.security_name, "ACME");
    assert_eq!(buy.price, 12.5);
    assert_eq!(buy.quantity, 10.0);
    assert_eq!(buy.commission_cost, 9.95);
    assert_eq!(buy.amount, 134.95);
    assert_eq!(buy.cleared_status, "R");
    assert_eq!(buy.memo, "Opening lot");
    assert_eq!(buy.description, "From checking");
    assert_eq!(buy.category, "[Checking]");
    assert_eq!(buy.amount_transferred, 134.95);

    let sell = &result.investments[1];
    assert_eq!(sell.account, "Brokerage");
    assert_eq!(sell.action, "Sell");
    assert_eq!(sell.amount, 50.0);
    assert_eq!(sell.date, "2020-03-05");

    // The register switches back from investments to the checking account.
    let refund = &result.transactions[1];
    assert_eq!(refund.account, "Checking");
    assert_eq!(refund.date, "2020-03-06");
    assert_eq!(refund.amount, 20.0);
    assert_eq!(refund.payee, "Refund");
}
