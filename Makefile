.PHONY: run build release clean test check

run:
	cargo run

build:
	cargo build

release:
	cargo build --release

check:
	cargo check

test:
	cargo test

clean:
	cargo clean
