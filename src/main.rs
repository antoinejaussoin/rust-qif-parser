use ::qif_parser::parse;
use std::fs;

fn main() {
    println!("QIF Parser Example");
    let content = fs::read_to_string("data/cic.qif").unwrap();
    let parsed = parse(&content, "%d/%m/%y").unwrap();
    println!("Type: {}", parsed.file_type);
    for item in &parsed.items {
        println!("{} {} {}", item.date, item.amount, item.payee);
    }
    let sum: f64 = parsed.items.iter().map(|item| item.amount).sum();
    println!("Format: {}", parsed);
    println!("Account balance: {}", sum);
    println!("To JSON: ");
    let json = serde_json::to_string(&parsed).unwrap();
    println!("{}", json);
}
