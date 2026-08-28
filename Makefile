.PHONY: run build release clean test check web-install web-build web-dev dev

# --- Backend ---------------------------------------------------------------
run:
	cargo run

build: web-build
	cargo build

release: web-build
	cargo build --release

check:
	cargo check

test:
	cargo test

clean:
	cargo clean
	rm -rf web/dist

# --- Web frontend ----------------------------------------------------------
web-install:
	npm --prefix web install

web-build:
	npm --prefix web run build

web-dev:
	npm --prefix web run dev

# Run backend and Vite dev server together. Use `make web-build` first when
# testing the production-serving path through the Rust binary.
dev:
	@echo "Starting backend on :7777 and Vite dev server on :5173"
	cargo run &
	@trap "kill $$!" EXIT; \
	  npm --prefix web run dev
