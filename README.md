# QIF Parser

Very high performance QIF (Quicken Interchange Format) parser in Rust.

## What is QIF?

QIF is a format invented by Quicken to record financial data.

You can read more on [this Wikipedia article](https://en.wikipedia.org/wiki/Quicken_Interchange_Format).

## What does this library do?

This library will take your QIF data as a string, parse it, and return some structured data for further processing.

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

https://rust-lang.github.io/api-guidelines/checklist.html

https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html

## Change Log

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
