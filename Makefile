build:
	cargo build

watch:
	cargo watch -x test

bench:
	cargo bench

compare:
	@echo "This will compare the speed of a Node.JS implementation versus this Rust implementation on the exact same file"
	@echo "Installing and compiling both versions..."
	@cd node; npm i
	@cargo build --release
	@echo "Executing both"
	@node ./node/index.js
	@./target/release/qif_parser
