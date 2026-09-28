use qif_parser::parse;

fn parse_ok<'a>(content: &'a str, date_format: &str) -> qif_parser::qif::Qif<'a> {
    let explicit =
        parse(content, date_format).unwrap_or_else(|err| panic!("parse failed: {}", err.details));
    let automatic = parse(content, None)
        .unwrap_or_else(|err| panic!("automatic date format failed: {}", err.details));
    assert_eq!(explicit, automatic);
    explicit
}

#[test]
fn empty_input_and_a_header_with_no_records() {
    let empty = parse_ok("", "%m/%d/%Y");
    assert_eq!(empty.file_type, "");
    assert!(empty.transactions.is_empty());

    let header = parse_ok("!Type:Bank\n", "%m/%d/%Y");
    assert_eq!(header.file_type, "Bank");
    assert!(header.transactions.is_empty());

    let blanks = parse_ok("\n\n  \n!Type:Cash\n\n", "%m/%d/%Y");
    assert_eq!(blanks.file_type, "Cash");
    assert!(blanks.transactions.is_empty());
}

#[test]
fn a_register_with_no_type_header_is_still_a_list_of_transactions() {
    let result = parse_ok("D1/2/2020\nT5.25\nPNo Header\n^\n", "%m/%d/%Y");
    assert_eq!(result.file_type, "");
    assert_eq!(result.transactions.len(), 1);
    assert_eq!(result.transactions[0].date, "2020-01-02");
    assert_eq!(result.transactions[0].amount, 5.25);
    assert_eq!(result.transactions[0].payee, "No Header");
}

#[test]
fn the_last_record_is_kept_when_the_closing_caret_is_missing() {
    let result = parse_ok("!Type:Bank\nD1/2/2020\nT12.50\nPShop\n", "%m/%d/%Y");
    assert_eq!(result.transactions.len(), 1);
    assert_eq!(result.transactions[0].payee, "Shop");
    assert_eq!(result.transactions[0].amount, 12.5);
    assert_eq!(result.transactions[0].date, "2020-01-02");
}

#[test]
fn a_new_header_closes_the_record_that_was_still_open() {
    let result = parse_ok(
        "\
!Type:Bank
D31/1/2020
T5
POne
!Type:Cash
D2/1/2020
T6
PTwo
^
",
        "%d/%m/%Y",
    );
    assert_eq!(result.file_type, "Cash");
    assert_eq!(result.transactions.len(), 2);
    assert_eq!(result.transactions[0].payee, "One");
    assert_eq!(result.transactions[0].date, "2020-01-31");
    assert_eq!(result.transactions[1].payee, "Two");
    assert_eq!(result.transactions[1].date, "2020-01-02");
}

#[test]
fn account_types_that_share_the_bank_field_layout() {
    for file_type in ["Cash", "CCard", "Oth A", "Oth L", "Oth S", "Tax", "Bill"] {
        let content = format!("!Type:{file_type}\nD1/1/2020\nT1\nP{file_type}\n^\n");
        let result = parse_ok(&content, "%m/%d/%Y");
        assert_eq!(result.file_type, file_type);
        assert_eq!(result.transactions.len(), 1);
        assert_eq!(result.transactions[0].payee, file_type);
        assert!(result.investments.is_empty());
    }
}

#[test]
fn type_headers_are_matched_without_regard_to_case() {
    let result = parse_ok("!type:cash\nD1/1/2020\nT3\nPLower\n^\n", "%m/%d/%Y");
    assert_eq!(result.file_type, "cash");
    assert_eq!(result.transactions.len(), 1);
    assert_eq!(result.transactions[0].amount, 3.0);

    let upper = parse_ok("!TYPE:CASH\nD1/1/2020\nT4\nPUpper\n^\n", "%m/%d/%Y");
    assert_eq!(upper.file_type, "CASH");
    assert_eq!(upper.transactions[0].payee, "Upper");
}

#[test]
fn an_investment_section_does_not_swallow_a_later_bank_section() {
    let result = parse_ok(
        "\
!Type:Invst
D1/1/2020
NBuy
YACME
T10
^
!Type:Bank
D2/1/2020
T-3.5
PCafe
^
!Type:Port
D3/1/2020
NDiv
YACME
T1.25
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.file_type, "Port");
    assert_eq!(result.investments.len(), 2);
    assert_eq!(result.investments[0].action, "Buy");
    assert_eq!(result.investments[1].action, "Div");
    assert_eq!(result.investments[1].amount, 1.25);
    assert_eq!(result.transactions.len(), 1);
    assert_eq!(result.transactions[0].payee, "Cafe");
    assert_eq!(result.transactions[0].amount, -3.5);
}

#[test]
fn unknown_lists_are_skipped_instead_of_becoming_transactions() {
    let result = parse_ok(
        "\
!Type:Payee
NSomebody
A1 Road
^
!Type:Invitem
NWidget
T12.00
^
!Type:Bank
D1/1/2020
T8
PReal
^
!NotAHeader
D2/1/2020
T9
PStill Real
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.transactions.len(), 2);
    assert_eq!(result.transactions[0].payee, "Real");
    assert_eq!(result.transactions[0].amount, 8.0);
    assert_eq!(result.transactions[1].payee, "Still Real");
    assert_eq!(result.transactions[1].amount, 9.0);
}

#[test]
fn byte_order_mark_crlf_and_old_mac_line_endings() {
    let crlf = parse_ok(
        "\u{feff}!Type:Bank\r\nD1/2/2020\r\nT1\r\nPWin\r\n^\r\n",
        "%m/%d/%Y",
    );
    assert_eq!(crlf.transactions.len(), 1);
    assert_eq!(crlf.transactions[0].payee, "Win");
    assert_eq!(crlf.transactions[0].date, "2020-01-02");

    let cr = parse_ok("!Type:Bank\rD1/2/2020\rT2\rPMac\r^\r", "%m/%d/%Y");
    assert_eq!(cr.transactions.len(), 1);
    assert_eq!(cr.transactions[0].payee, "Mac");
    assert_eq!(cr.transactions[0].amount, 2.0);
}

#[test]
fn comments_blank_lines_and_a_leading_multibyte_character_are_ignored() {
    let result = parse_ok(
        "\
#TEF VERSION 1.01
!Type:Bank

# a note between fields
D1/2/2020
😀not a field
T1.5
POk
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.transactions.len(), 1);
    assert_eq!(result.transactions[0].payee, "Ok");
    assert_eq!(result.transactions[0].amount, 1.5);
    assert_eq!(result.transactions[0].date, "2020-01-02");
}

#[test]
fn duplicate_fields_keep_the_later_value_including_u_and_t() {
    let later_amount = parse_ok(
        "!Type:Bank\nD1/1/2020\nT10\nT20.5\nPFirst\nPSecond\n^\n",
        "%m/%d/%Y",
    );
    assert_eq!(later_amount.transactions[0].amount, 20.5);
    assert_eq!(later_amount.transactions[0].payee, "Second");

    let u_then_t = parse_ok(
        "!Type:Bank\nD1/1/2020\nU1,000.00\nT1000\nPPayee\n^\n",
        "%m/%d/%Y",
    );
    assert_eq!(u_then_t.transactions[0].amount, 1_000.0);

    let t_then_u = parse_ok("!Type:Bank\nD1/1/2020\nT10\nU20\nPPayee\n^\n", "%m/%d/%Y");
    assert_eq!(t_then_u.transactions[0].amount, 20.0);

    let noisy = parse_ok(
        "!Type:Invst\nD1/1/2020\nNCash\nU-0.2800000011920929\nT-0.2800000011920929\n^\n",
        "%m/%d/%Y",
    );
    assert_eq!(noisy.investments[0].amount, -0.2800000011920929);
}

#[test]
fn an_empty_memo_is_legal_and_addresses_stay_in_order() {
    let result = parse_ok(
        "\
!Type:Bank
D1/1/2020
T1
PPayee
M
Aone
Atwo
A
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.transactions[0].memo, "");
    assert_eq!(result.transactions[0].address, vec!["one", "two", ""]);
}

#[test]
fn check_numbers_before_a_split_stay_on_the_transaction() {
    let result = parse_ok(
        "\
!Type:Bank
D1/1/2020
T-12
N100
NATM
SFood
$-5
N200
SRent
$-7
N300
^
",
        "%m/%d/%Y",
    );
    let txn = &result.transactions[0];
    // The second N, still before any split, overwrites the transaction number.
    assert_eq!(txn.number_of_the_check, "ATM");
    assert_eq!(txn.splits[0].number_of_the_check, "200");
    assert_eq!(txn.splits[1].number_of_the_check, "300");
    assert_eq!(txn.splits[0].amount, -5.0);
    assert_eq!(txn.splits[1].category, "Rent");
}

#[test]
fn reference_numbers_are_stored_as_written() {
    let result = parse_ok(
        "\
!Type:Bank
D1/1/2020
T1
NDeposit
PIn
^
D1/1/2020
T-1
NEFT
POut
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.transactions[0].number_of_the_check, "Deposit");
    assert_eq!(result.transactions[1].number_of_the_check, "EFT");
}

#[test]
fn split_percentages_can_sit_beside_amounts() {
    let result = parse_ok(
        "\
!Type:Bank
D1/1/2020
T-20
SFood
Egroceries
%50
$-10
SRent
%50.5
^
",
        "%m/%d/%Y",
    );
    let splits = &result.transactions[0].splits;
    assert_eq!(splits[0].memo, "groceries");
    assert_eq!(splits[0].percentage, 50.0);
    assert_eq!(splits[0].amount, -10.0);
    assert_eq!(splits[1].percentage, 50.5);
    assert_eq!(splits[1].amount, 0.0);
}

#[test]
fn a_split_detail_with_no_split_is_an_error() {
    for field in ["Ememo", "$10", "%50"] {
        let content = format!("!Type:Bank\nD1/1/2020\nT1\n{field}\n^\n");
        let err = parse(&content, "%m/%d/%Y").unwrap_err();
        assert_eq!(err.details, "There should be a split item here");
    }
}

#[test]
fn cleared_status_values_are_kept_verbatim() {
    let result = parse_ok(
        "\
!Type:Bank
D1/1/2020
T1
C*
PStar
^
D1/1/2020
T1
CX
PReconciled
^
D1/1/2020
T1
CR
PAlso reconciled
^
D1/1/2020
T1
Cc
PCleared
^
D1/1/2020
T1
C?
PBudgeted
^
D1/1/2020
T1
C!
PAlso budgeted
^
",
        "%m/%d/%Y",
    );
    let statuses: Vec<&str> = result
        .transactions
        .iter()
        .map(|txn| txn.cleared_status)
        .collect();
    assert_eq!(statuses, vec!["*", "X", "R", "c", "?", "!"]);
}

#[test]
fn categories_classes_and_transfers_stay_on_one_line() {
    let result = parse_ok(
        "\
!Type:Bank
D1/1/2020
T-1
LFood:Groceries/Vacation
PShop
^
D1/1/2020
T-1
L[Checking]/Vacation
PMove
^
!Type:Invst
D1/1/2020
NMiscExpX
T1000.00
Lexpense category/expense class|[Transfer account]/transfer class
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.transactions[0].category, "Food:Groceries/Vacation");
    assert_eq!(result.transactions[1].category, "[Checking]/Vacation");
    assert_eq!(
        result.investments[0].category,
        "expense category/expense class|[Transfer account]/transfer class"
    );
    assert_eq!(result.investments[0].action, "MiscExpX");
}

#[test]
fn reimbursable_flag_and_amounts_with_currency_or_european_spelling() {
    let result = parse_ok(
        "\
!Type:Bank
D1/1/2020
T$1,234.50
PDollars
F
^
D1/1/2020
T(99.95)
PParens
Fyes
^
D1/1/2020
T1.234,56
PEurope
^
D1/1/2020
T10'000.00
POld Quicken
^
D1/1/2020
T -8.5
PSpaced
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.transactions[0].amount, 1_234.5);
    assert!(result.transactions[0].reimbursable);
    assert_eq!(result.transactions[1].amount, -99.95);
    assert!(result.transactions[1].reimbursable);
    assert_eq!(result.transactions[2].amount, 1_234.56);
    assert!(!result.transactions[2].reimbursable);
    assert_eq!(result.transactions[3].amount, 10_000.0);
    assert_eq!(result.transactions[4].amount, -8.5);
}

#[test]
fn a_bad_amount_or_date_fails_the_whole_file() {
    let amount = parse("!Type:Bank\nD1/1/2020\nTInvst\n^\n", "%m/%d/%Y").unwrap_err();
    assert_eq!(
        amount.details,
        "Could not parse the following as a number: 'Invst'"
    );

    let date = parse("!Type:Bank\nD27/08/2018\nT1\n^\n", "%m/%d/%Y").unwrap_err();
    assert_eq!(
        date.details,
        "Error when parsing date: input is out of range 27/08/2018"
    );
    let guessed = parse("!Type:Bank\nD27/08/2018\nT1\n^\n", None).unwrap();
    assert_eq!(guessed.transactions[0].date, "2018-08-27");

    let mixed = parse("!Type:Bank\nD13/01/2020\nT1\n^\nD01/13/2020\nT2\n^\n", None).unwrap_err();
    assert_eq!(
        mixed.details,
        "Could not guess the date format: some dates are day-first and some are month-first"
    );

    let price = parse("!Type:Prices\nnot a price\n^\n", "%m/%d/%y").unwrap_err();
    assert!(price.details.contains("not a price"));
}

#[test]
fn investment_actions_commission_fractional_price_and_a_repeated_caret() {
    let result = parse_ok(
        "\
!Type:Invst
D1/1/2020
NKauf
YACME
I10 1/2
Q4,896.201
O14.95
T100.00
PReminder
Mnote
CX
^
D2/1/2020
NStkSplit
YACME
Q2
^
D3/1/2020
NReinvDiv
YACME
T3
^
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.investments.len(), 3);
    let buy = &result.investments[0];
    assert_eq!(buy.action, "Kauf");
    assert_eq!(buy.price, 10.5);
    assert_eq!(buy.quantity, 4_896.201);
    assert_eq!(buy.commission_cost, 14.95);
    assert_eq!(buy.amount, 100.0);
    assert_eq!(buy.description, "Reminder");
    assert_eq!(buy.memo, "note");
    assert_eq!(buy.cleared_status, "X");

    assert_eq!(result.investments[1].action, "StkSplit");
    assert_eq!(result.investments[1].quantity, 2.0);
    assert_eq!(result.investments[2].action, "ReinvDiv");
    assert_eq!(result.investments[2].amount, 3.0);
}

#[test]
fn invoice_fields_and_description_continuations() {
    let result = parse_ok(
        "\
!Type:Invoice
D6/1/2020
T165.40
PCustomer
XI1
XE6/17/2020
XC[*Sales Tax*]
XR7.70
XT15.40
XPPO-9
XMThanks
XAATTN: Receiving
XAWarehouse
XU3
XD7/1/2020
XY10.00
XD8/1/2020
XY10.50
XSRed shoes
with white laces
XNSHOES
XKApparel
X#2
X$75.00
XFT
XSBlue hat
Tthis is still the description
XNHats
X#1
X$15.00
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.file_type, "Invoice");
    assert_eq!(result.transactions.len(), 1);
    let txn = &result.transactions[0];
    assert_eq!(txn.date, "2020-06-01");
    assert_eq!(txn.amount, 165.4);
    assert_eq!(txn.payee, "Customer");
    // The T line after XS is a continuation of the description, not a new amount.
    assert_eq!(txn.amount, 165.4);

    let invoice = txn.invoice.as_ref().unwrap();
    assert_eq!(invoice.kind, "1");
    assert_eq!(invoice.due_date, "2020-06-17");
    assert_eq!(invoice.tax_account, "[*Sales Tax*]");
    assert_eq!(invoice.tax_rate, 7.7);
    assert_eq!(invoice.tax_amount, 15.4);
    assert_eq!(invoice.po_number, "PO-9");
    assert_eq!(invoice.customer_message, "Thanks");
    assert_eq!(
        invoice.shipping_address,
        vec!["ATTN: Receiving", "Warehouse"]
    );
    assert_eq!(invoice.payment_count, 3.0);
    assert_eq!(
        invoice.payment_dates,
        vec!["2020-07-01".to_string(), "2020-08-01".to_string()]
    );
    assert_eq!(invoice.payment_amounts, vec![10.0, 10.5]);
    assert_eq!(invoice.lines.len(), 2);
    assert_eq!(
        invoice.lines[0].description,
        vec!["Red shoes", "with white laces"]
    );
    assert_eq!(invoice.lines[0].category, "SHOES");
    assert_eq!(invoice.lines[0].class_name, "Apparel");
    assert_eq!(invoice.lines[0].quantity, 2.0);
    assert_eq!(invoice.lines[0].unit_price, 75.0);
    assert!(invoice.lines[0].taxable);
    assert_eq!(
        invoice.lines[1].description,
        vec!["Blue hat", "Tthis is still the description"]
    );
    assert_eq!(invoice.lines[1].category, "Hats");
    assert_eq!(invoice.lines[1].quantity, 1.0);
    assert_eq!(invoice.lines[1].unit_price, 15.0);
    assert!(!invoice.lines[1].taxable);
}

#[test]
fn memorized_checks_loans_and_investments_are_not_register_transactions() {
    let result = parse_ok(
        "\
!Type:Memorized
T-50.00
POakwood Gardens
MRent
LHousing
KC
^
T-1000
PBig Bank
L[Mortgage]
101/15/2000
230
360
412
57.25
6150000
7200000
KP
^
D1/2/2020
NBuy
YIBM
I10
Q2
T20
KI
^
",
        "%m/%d/%Y",
    );
    assert!(result.transactions.is_empty());
    assert!(result.investments.is_empty());
    assert_eq!(result.memorized.len(), 3);

    let check = &result.memorized[0];
    assert_eq!(check.kind, "C");
    assert!(check.investment.is_none());
    assert_eq!(check.transaction.amount, -50.0);
    assert_eq!(check.transaction.payee, "Oakwood Gardens");
    assert_eq!(check.transaction.memo, "Rent");
    assert_eq!(check.transaction.category, "Housing");

    let loan = &result.memorized[1];
    assert_eq!(loan.kind, "P");
    assert_eq!(loan.transaction.amount, -1_000.0);
    assert_eq!(loan.transaction.payee, "Big Bank");
    assert_eq!(loan.transaction.category, "[Mortgage]");
    assert_eq!(loan.first_payment_date, "2000-01-15");
    assert_eq!(loan.loan_years, 30.0);
    assert_eq!(loan.payments_made, 60.0);
    assert_eq!(loan.periods_per_year, 12.0);
    assert_eq!(loan.interest_rate, 7.25);
    assert_eq!(loan.current_balance, 150_000.0);
    assert_eq!(loan.original_amount, 200_000.0);

    let buy = &result.memorized[2];
    assert_eq!(buy.kind, "I");
    assert_eq!(buy.transaction.payee, "");
    let investment = buy.investment.as_ref().unwrap();
    assert_eq!(investment.date, "2020-01-02");
    assert_eq!(investment.action, "Buy");
    assert_eq!(investment.security_name, "IBM");
    assert_eq!(investment.price, 10.0);
    assert_eq!(investment.quantity, 2.0);
    assert_eq!(investment.amount, 20.0);
}

#[test]
fn a_date_with_a_space_after_the_code_still_parses() {
    let result = parse_ok("!Type:Bank\nD 1/2/2020\nT1\nPPayee\n^\n", "%m/%d/%Y");
    assert_eq!(result.transactions[0].date, "2020-01-02");
}

#[test]
fn trailing_spaces_on_a_payee_are_preserved() {
    let result = parse_ok("!Type:Bank\nD1/1/2020\nT1\nPFOO     BAR\n^\n", "%m/%d/%Y");
    assert_eq!(result.transactions[0].payee, "FOO     BAR");
}

#[test]
fn an_option_between_fields_does_not_split_the_transaction() {
    let result = parse_ok(
        "\
!Type:Bank
D1/1/2020
!Option:AllXfr
T5
PPayee
^
",
        "%m/%d/%Y",
    );
    assert_eq!(result.options, vec!["AllXfr"]);
    assert_eq!(result.transactions.len(), 1);
    assert_eq!(result.transactions[0].amount, 5.0);
    assert_eq!(result.transactions[0].payee, "Payee");
    assert_eq!(result.transactions[0].date, "2020-01-01");
}
