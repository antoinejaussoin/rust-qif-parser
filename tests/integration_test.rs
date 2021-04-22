use ::qif_parser::parse;
use std::fs;

#[test]
fn test_wikipedia_simple_example() {
    let content = fs::read_to_string("data/wikipedia_simple.qif").unwrap();
    let result = parse(&content, "%m/%d/%y").unwrap();
    assert!(content.len() > 0);
    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert_eq!(result.items.len(), 3);

    // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2010-03-03");
    assert_eq!(first.amount, -379.0);
    assert_eq!(first.category, "");
    assert_eq!(first.payee, "CITY OF SPRINGFIELD");
    assert_eq!(first.cleared_status, "");

    // Second items
    let second = &result.items[1];
    assert_eq!(second.date, "2010-03-04");
    assert_eq!(second.amount, -20.28);
    assert_eq!(second.category, "");
    assert_eq!(second.payee, "YOUR LOCAL SUPERMARKET");
    assert_eq!(second.cleared_status, "");
}

#[test]
fn test_wikipedia_investment_example() {
    let content = fs::read_to_string("data/wikipedia_investments.qif").unwrap();
    let result = parse(&content, "%m/%d'%y").unwrap();
    assert!(content.len() > 0);

    // QIF metadata
    assert_eq!(result.file_type, "Invst");

    // Counting items
    assert_eq!(result.items.len(), 0);
    assert_eq!(result.investments.len(), 2);

    // First items
    let first = &result.investments[0];
    assert_eq!(first.date, "2007-12-21");
    assert_eq!(first.action, "Buy");
    assert_eq!(first.security_name, "IBM");
    assert_eq!(first.amount, 11010.00);
    assert_eq!(first.price, 110.10);
    assert_eq!(first.quantity, 100.0);
    assert_eq!(
        first.memo,
        "Purchase of 100 shares of IBM stock on 21 December 2007 at $110.10 per share"
    );
}

#[test]
fn test_wikipedia_example() {
    let content = fs::read_to_string("data/wikipedia.qif").unwrap();
    let result = parse(&content, "%m/%d'%Y").unwrap();
    assert!(content.len() > 0);
    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert_eq!(result.items.len(), 6);

    // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2020-02-10");
    assert_eq!(first.amount, 0.0);
    assert_eq!(first.category, "[TestExport]");
    assert_eq!(first.payee, "Opening Balance");
    assert_eq!(first.cleared_status, "X");

    // Second item (splits)
    let second = &result.items[1];
    assert_eq!(second.splits.len(), 2);
    assert_eq!(second.splits[0].category, "Bills:Cell Phone");
    assert_eq!(second.splits[0].memo, "sign up credit");
    assert_eq!(second.splits[0].amount, -15.0);
    assert_eq!(second.splits[1].category, "Bills:Cell Phone");
    assert_eq!(second.splits[1].memo, "new account");
    assert_eq!(second.splits[1].amount, 82.50);

    // Third item (memo)
    let third = &result.items[2];
    assert_eq!(third.memo, "money back for damaged parcel");
    assert_eq!(third.category, "Miscellaneous");
}

#[test]
fn test_monzo_example() {
    let content = fs::read_to_string("data/monzo.qif").unwrap();
    let result = parse(&content, "%d/%m/%Y").unwrap();
    assert!(content.len() > 0);

    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert_eq!(result.items.len(), 13);

    // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2018-08-27");
    assert_eq!(first.amount, 1000.0);
    assert_eq!(first.category, "");
    assert_eq!(first.payee, "Jane Doe");
    assert_eq!(first.cleared_status, "");

    // Third items
    let third = &result.items[2];
    assert_eq!(third.date, "2018-08-28");
    assert_eq!(third.amount, -15.0);
    assert_eq!(third.category, "Transport");
    assert_eq!(third.payee, "Infinity Motor Cycles");
    assert_eq!(third.address[0], "30-32 FairyLand High Street");
}

#[test]
fn test_cic_example() {
    let content = fs::read_to_string("data/cic.qif").unwrap();
    let result = parse(&content, "%d/%m/%y").unwrap();
    assert!(content.len() > 0);

    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert_eq!(result.items.len(), 12);

    // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2020-05-19");
    assert_eq!(first.amount, 500.0);
    assert_eq!(first.category, "");
    assert_eq!(first.payee, "REM CHQ REF1234");
    assert_eq!(first.cleared_status, "");

    // Third items
    let third = &result.items[2];
    assert_eq!(third.date, "2020-06-02");
    assert_eq!(third.amount, -9.59);
    assert_eq!(third.category, "");
    assert_eq!(third.payee, "KIMSUFI CARTE 1234 PAIEMENT CB 0106 ROUBAIX");
    assert_eq!(third.address.len(), 0);
}

#[test]
fn test_nasty_example() {
    let content = fs::read_to_string("data/nasty.qif").unwrap();
    let result = parse(&content, "%d/%m/%Y").unwrap();
    assert!(content.len() > 0);

    // QIF metadata
    assert_eq!(result.file_type, "Bank");

    // Items
    assert_eq!(result.items.len(), 6);

    // First items
    let first = &result.items[0];
    assert_eq!(first.date, "2018-08-27");
    assert_eq!(first.amount, 10_000.0);
    assert_eq!(first.category, "");
    assert_eq!(first.payee, "Jane Doe");
    assert_eq!(first.cleared_status, "");

    // Second items
    let second = &result.items[1];
    assert_eq!(second.date, "2018-08-27");
    assert_eq!(second.amount, -10_000_000.0);
    assert_eq!(second.category, "Shopping");
    assert_eq!(second.payee, "Huge Amount 😅 with UTF8");
    assert_eq!(second.address.len(), 3);
    assert_eq!(second.address[0], "Address line 1");
    assert_eq!(second.address[1], "Address line 2");
    assert_eq!(second.address[2], "Address line 3");

    // Item with + sign on amount
    let third = &result.items[2];
    assert_eq!(third.amount, 123.0);

    // Item with missing leading 0 on date
    let fourth = &result.items[3];
    assert_eq!(fourth.date, "2018-08-28");

    // Item with cheque number
    let fifth = &result.items[4];
    assert_eq!(fifth.number_of_the_check, "CHQ100");

    // Item with cheque number on the splits
    let sixth = &result.items[5];
    assert_eq!(sixth.number_of_the_check, "");
    assert_eq!(sixth.splits[0].number_of_the_check, "CHQ101");
    assert_eq!(sixth.splits[1].number_of_the_check, "CHQ102");
}

#[test]
fn test_errors() {
    let content = fs::read_to_string("data/monzo.qif").unwrap();
    match parse(&content, "%m/%d/%Y") {
        Err(err) => assert_eq!(
            err.details,
            "Error when parsing date: input is out of range 27/08/2018"
        ),
        Ok(_) => panic!("It should have failed"),
    };
}
