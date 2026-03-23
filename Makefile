.PHONY: build release install test lint fmt clean

build:
	cargo build

release:
	cargo build --release

install:
	cargo install --path .

test:
	cargo nextest run

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

clean:
	cargo clean
