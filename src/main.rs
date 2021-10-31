use ::qif_parser::parse;
use std::time::Instant;

fn main() {
    let item = "D02/10/2020
C*
Mtest order 1
T-100.00
PAmazon.com
LFood:Groceries
SFood:Groceries
E50%
$-50.00
STransportation:Automobile
E25%
$-25.00
SPersonal Care:Haircare
E10%
$-10.00
SHealthcare:Prescriptions
E15%
$-15.00
^
";
    let size = 10_000;
    let mut full = String::with_capacity(item.len() * size);
    for _ in 0..10_000 {
        full.push_str(item);
    }
    let before = Instant::now();
    let parsed = parse(&full, "%d/%m/%Y").unwrap();
    let elapsed = before.elapsed();
    println!(
        "RUST: Done processing {} items. Time it would take to process 1M items: {}ms",
        parsed.transactions.len(),
        elapsed.as_millis() * 100
    );
}
