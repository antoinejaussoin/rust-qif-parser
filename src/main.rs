mod date;
mod parser;

use std::fs;

fn main() {
    println!("QIF Parser test");
    // let content =
    //     fs::read_to_string("/Users/antoine/dev/rust-qif-parser/src/data/example2.qif").unwrap();
    let content =
        fs::read_to_string("/Users/antoine/dev/rust-qif-parser/src/example1.qif").unwrap();
    let parsed = parser::parse_with_format(&content, "%m/%d'%Y");
    // let parsed = parser::parse_with_format(&content, "%d/%m/%Y");
    println!("Type: {}", parsed.file_type);
    for item in &parsed.items {
        println!("{} {} {}", item.date, item.amount, item.payee);
    }
    let sum: f32 = parsed.items.iter().map(|item| item.amount).sum();
    println!("Account balance: {}", sum);
}
