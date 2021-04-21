use ::qif_parser::parse;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let item = "D2/10'2020
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
    c.bench_function("Small", |b| b.iter(|| parse(black_box(&full), "%m/%d'%Y")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
