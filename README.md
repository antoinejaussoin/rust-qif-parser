# QIF Parser

QIF (Quicken Interchange Format) parser in Rust

## What is QIF?

QIF is a format invented by Quicken to record financial data.

You can read more on [this Wikipedia article](https://en.wikipedia.org/wiki/Quicken_Interchange_Format).

## What does this library do?

This library will take your QIF data as a string, parse it, and return some structured data for further processing.

## Various links

https://en.wikipedia.org/wiki/Quicken_Interchange_Format

https://rust-lang.github.io/api-guidelines/checklist.html

https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html

## Change Log

### Version 0.1.0
- Make the code more Rusty (using match instead of if-statements)
- Support for all the QIF fields as defined in the Wikipedia entry
- More tests
- Return &str instead of String on the returned object (except for the date)

### Version 0.0.6

- Use `f64` instead of `f32`
