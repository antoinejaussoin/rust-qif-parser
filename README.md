# QIF Parser

Very high performance QIF (Quicken Interchange Format) parser in Rust.

## Usage

```toml
qif_parser = "0.6"
```

```rust
use qif_parser::parse;

fn main() -> Result<(), qif_parser::errors::QifParsingError> {
    let qif = "\
!Type:Bank
D02/10/2020
T-100.00
PAmazon.com
LFood:Groceries
^
";

    let parsed = parse(qif, "%d/%m/%Y")?;
    let first = &parsed.transactions[0];
    println!("{} {} {}", first.date, first.amount, first.payee);
    Ok(())
}
```

`parse` takes the file text and a [chrono date format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers). Dates come back as `YYYY-MM-DD`, so this example prints `2020-10-02 -100 Amazon.com`.

## What is QIF?

QIF is a format invented by Quicken to record financial data.

You can read more on [this Wikipedia article](https://en.wikipedia.org/wiki/Quicken_Interchange_Format).

## What does this library do?

Pass the text of a QIF file and a [chrono date format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers). You get a `Qif` value back: transactions, investments, and the other lists Quicken puts in the same file (accounts, categories, classes, tags, securities, prices, memorized transactions).

QIF does not define one date format or one way to write an amount, and a single file can switch register type in the middle. The parser follows the section it is in, so `!Account` / `TBank` is an account type, a category's `T` line is a tax flag, and a later `!Type:Bank` is a bank register again after an investment section.

## How thoroughly is it tested?

`cargo test` runs 60 tests. They are split four ways:

- `tests/integration_test.rs` holds the original bank-file fixtures. Those tests are unchanged and still pass.
- `tests/spec_examples.rs` parses samples copied from published QIF descriptions.
- `tests/edge_cases.rs` covers format quirks, broken records, and section changes.
- Unit tests in `src/date.rs` and `src/number.rs` cover date and amount spelling on their own.

The fixtures live in `data/`. The bank exports, the Wikipedia samples, `data/external/`, and `data/gnucash_investments.qif` are copied from those sources. `data/prices.qif`, `data/lists.qif`, and `data/autoswitch.qif` are built from the same published layout and checked field by field. The assertions check parsed values, not just that parsing returned `Ok`.

### Real bank exports

These were already in the crate, and they still parse exactly as before:

| File | What the test checks |
| --- | --- |
| `data/monzo.qif` | 13 UK bank transactions, categories, and payee addresses |
| `data/cic.qif` | 12 French bank transactions, two-digit years (`%d/%m/%y`) |
| `data/amex.qif` | A credit-card file (`!Type:CCard`) whose blank lines must not become transactions, plus a long foreign-spend memo |
| `data/nasty.qif` | `10,000.00` and `-10,000,000.00`, a UTF-8 payee, a leading `+`, a date with no leading zero (`28/8/2018`), and cheque numbers on the transaction and on each split |
| `data/wikipedia.qif`, `wikipedia_simple.qif`, `wikipedia_investments.qif` | The samples from the Wikipedia article, including splits and `!Type:Invst` |

A wrong date format fails with a stable message. Parsing the Monzo file as `%m/%d/%Y` returns `Error when parsing date: input is out of range 27/08/2018`.

### Published specifications

**GnuCash's own QIF notes.** `data/gnucash_investments.qif` is the investment sample from [GnuCash's `file-format.txt`](https://github.com/Gnucash/gnucash/blob/stable/gnucash/import-export/qif-imp/file-format.txt). The `T` line under `!Account` is the account type, not an amount:

```
!Account
NAssets:Investments:Mutual Fund
TInvst
^
!Type:Invst
D10/30/2006
Q0.9
T500
PPurchase
NBuyX
L[Assets:Investments:Mutual Fund:Cash]
YFOO
^
```

The test expects one account named `Assets:Investments:Mutual Fund`, no bank transactions, and a `BuyX` of 0.9 shares of `FOO` for 500 on 2006-10-30, with the cash account in brackets kept as the category. The same file's `SellX` is checked the same way. Both investments are tagged with that account name.

**Intuit's examples, via the Qif gem.** `data/external/` is copied from the fixtures of [jemmyw/Qif](https://github.com/jemmyw/Qif/tree/master/spec/fixtures), which follow Intuit's QIF write-up. `splits.qif` is a good illustration of how loose those files are:

```
!Type:Bank
D6/1/94
T-1,000.00
CX
N1005
PBank Of Mortgage
MMemo
L[linda]
S[steve]
ECash
$-253.64
SMort Int
$=746.36
^
D6/2/94
Scategory 1
$23
T75.00
PDeposit
^
```

The test checks both records. The first has two splits: `[steve]` / `Cash` / `-253.64`, then `Mort Int` / `746.36`. The `=` in `$=746.36` is kept as a positive amount, which is how that sample is written. The second record opens with the split and only then gives the transaction amount, and the test expects amount `75`, payee `Deposit`, and one split of `23` in `category 1`.

The same suite checks Intuit's investment sample (`ShrsIn` of `ibm4`, then a `BuyX` with `L[CHECKING]` and `$100.00` as the amount transferred) and six address lines, the last three of them blank.

Two more fixtures from that gem pin down spelling. In `3_records_spaced.qif`, Quicken pads a single-digit day with a space:

```
D1/ 1/10
T-10.00
```

With `%m/%d/%y` that is 1 January 2010 and `-10`. The next record, `D6/ 1/10`, is 1 June 2010, and `D12/29/10` is 29 December 2010. In `3_records_separator.qif`, `T-1,010.00` is `-1010` and `T-30,020.00` is `-30020`. A file that puts a `^` immediately after `!Type:Bank` still has three transactions, not four.

**Security prices.** Price lines are not letter-coded. `data/prices.qif` follows the Intuit / GnuCash description, including a fractional price and a symbol that itself contains a comma:

```
!Type:Prices
"INTU",50 1/2,"6/30/98"
^
"FOO, BAR",1.25,"1/2/20"
^
INTC,10 3/8,3/17/11
^
```

Those parse as INTU at 50.5 on 1998-06-30, `FOO, BAR` at 1.25 on 2020-01-02, and INTC at 10.375 on 2011-03-17. `50.50` and `50 1/2` are the same price.

**Lists and a multi-account export.** `data/lists.qif` is a category, class, tag, security, and budget file. A category `T` line sets the tax-related flag. It does not try to parse `T` as a number, and `TStock` on a security is the security type. A category with neither `I` nor `E` is an expense, which is Quicken's rule. Budget rows stay separate records, with one amount per `B` line.

`data/autoswitch.qif` is the Quicken export shape: `!Option:AutoSwitch`, an account list (including a card's credit limit and statement balance), `!Clear:AutoSwitch`, then a bank register and a brokerage register. The grocery transaction is attached to `Checking`, the `Buy` (commission `O9.95`, transfer `$134.95`) to `Brokerage`, and a later refund is a bank transaction again rather than another investment. An extra `^` in that file does not create an empty investment.

Wikipedia's import example is tested the same way: `!Account` / `NJoint Brokerage Account` / `TInvst`, then `!Type:Invst` and the IBM purchase dated `12/21' 7`.

### Edge cases

`tests/edge_cases.rs` checks the corners that real files combine:

- A UTF-8 BOM, Windows `\r\n`, and old Mac `\r`-only files.
- A missing final `^`, and a new `!Type` header that closes the record still open above it.
- `!Type:Cash`, `CCard`, `Oth A`, `Oth L`, `Oth S`, `Invoice`, `Tax`, `Bill`, and `Port`, including `!type:cash` and `!TYPE:CASH`.
- `!Type:Payee` and `!Type:Invitem` are skipped, so their records are not dumped into `transactions`. An unknown `!NotAHeader` line does not end the current register.
- `#` comment lines, blank lines, and a line whose first character is an emoji.
- The last `T`, `U`, or `P` wins when the same field is repeated. `U1,000.00` followed by `T1000` is 1000.
- Cheque number `N` before the first split stays on the transaction. `N` after an `S` belongs to that split. `NDeposit` and `NEFT` are kept as written.
- A split can carry both `$` and `%`. `E`, `$`, or `%` before any `S` is an error: `There should be a split item here`.
- Cleared status is kept as written: `*`, `X`, `R`, `c`, `?`, `!`.
- Categories such as `Food:Groceries/Vacation`, `[Checking]/Vacation`, and a `MiscExpX` line with a `|` are left intact.
- Amounts: `$1,234.50`, `(99.95)`, `1.234,56`, `10'000.00` (older Quicken thousands separator), and a leading unicode minus.
- Investment actions `Kauf`, `StkSplit`, and `ReinvDiv`, a price of `10 1/2`, quantity `4,896.201`, and commission `O14.95`.
- Invoice `X` fields, including an `XS` description that continues onto the next line. A `T` line in that continuation stays part of the description.
- Memorized checks (`KC`), a loan with the seven amortization lines (`1` through `7`), and a memorized buy (`KI`). None of those are added to the register.

Dates are only as ambiguous as the format you pass. `D6/1/94` is June 1994 with `%m/%d/%y` and 6 January 1994 with `%d/%m/%y`. The tests pass the format explicitly and assert the resulting `YYYY-MM-DD` date. One amount rule is fixed because QIF itself is not: a comma followed by exactly three digits is a thousands separator (`1,000` is 1000, `10,000.00` is 10000), while `1,50` and `1.234,56` are European decimals.

## What about performance?

This repository compares the same functionality written in Node.JS and in Rust.
If you have both Node and Rust installed, you can run both by doing `make compare`.

Spoiler alert: for 1 million transaction items, the Node implementation would take about **4 minutes** on a M1 Mac, and the Rust implementation a little over... **1 second**. We then have a **200x** speed difference between the two. Fancy that!

Actual output from my M1 Mac:

```
Executing both
NODE: Done processing 1000 items. Time it would take to process 1M items: 238793ms
RUST: Done processing 100000 items. Time it would take to process 1M items: 1430ms
```

## Various links

https://en.wikipedia.org/wiki/Quicken_Interchange_Format

https://github.com/Gnucash/gnucash/blob/stable/gnucash/import-export/qif-imp/file-format.txt

https://github.com/jemmyw/Qif/tree/master/spec/fixtures

https://rust-lang.github.io/api-guidelines/checklist.html

https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html

## Change Log

### Version 0.6.0

- Parse every section of a Quicken file: accounts, categories, classes, tags, securities, prices, and memorized transactions, not only a single bank or investment register.
- Read investment commission and amount transferred, invoice lines, split percentages, and the account each record belongs to.
- Accept the amount and date spellings used by Quicken and by bank exports.
- BREAKING CHANGE: `Qif`, `QifTransaction`, `QifInvestment`, and `QifSplit` have new fields. Build them with `..Default::default()`, or name the new fields. `#[serde(default)]` is set on those fields, so older JSON still deserializes.

### Version 0.5.0

- Migrate to the Rust 2024 edition (Rust 1.98 or newer)
- Upgrade dependencies (chrono, serde, criterion)

### Version 0.4.0

- Upgrade dependencies

### Version 0.3.0

- Adding support for Amex QIF files, which include blank lines

### Version 0.2.0
- Implementing useful traits, such as debug, format, clone, serialize and deserialize.
- Adding Serde as a dependency (for the reason above)
- Moving files around so it's cleaner and not all the code is in lib.rs
- BREAKING CHANGE: the Qif object is now returning a "transactions" vec, not "items".
- Adding a benchmark comparison with Node.JS.

### Version 0.1.0
- Make the code more Rusty (using match instead of if-statements)
- Support for all the QIF fields as defined in the Wikipedia entry
- More tests
- Return &str instead of String on the returned object (except for the date). This should improve performance dramatically.
- Adding benchmark

### Version 0.0.6

- Use `f64` instead of `f32`
