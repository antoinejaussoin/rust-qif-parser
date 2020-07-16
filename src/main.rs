use ::qif_parser::parse;
use std::fs;

fn main() {
    println!("QIF Parser Example");
    // let content = fs::read_to_string("data/wikipedia.qif").unwrap();
    // let parsed = parse(&content, "%m/%d'%Y").unwrap();
    let content = fs::read_to_string("data/perso_cic.qif").unwrap();
    let parsed = parse(&content, "%d/%m/%y").unwrap();
    println!("Type: {}", parsed.file_type);
    for item in &parsed.items {
        println!("{} {} {}", item.date, item.amount, item.payee);
    }
    let sum: f32 = parsed.items.iter().map(|item| item.amount).sum();
    println!("Account balance: {}", sum);
}
