.PHONY: run build release clean test check web-install web-build web-dev dev tauri-dev tauri-build

# --- Backend ---------------------------------------------------------------
# Run the Tauri app without the CLI wrapper. Requires `web/dist` to exist;
# `make web-build` produces it. The Tauri runtime loads the bundled assets
# directly; this is the fastest path for CI or smoke tests.
run: web-build
	cargo run

# Plain cargo build; bundles the web/dist into the binary via tauri-build.
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

# --- Tauri CLI wrappers (require `cargo install tauri-cli --version "^2.0"`) -
tauri-dev:
	cargo tauri dev

tauri-build:
	cargo tauri build

# Default `make dev`: spawn Vite + cargo run in parallel. The Tauri runtime
# loads `web/dist`, so this uses the pre-built SPA (run `make web-build` once).
# For live HMR use `make tauri-dev` instead.
dev: web-build
	@echo "Running Tauri app on bundled web/dist (run `make tauri-dev` for HMR)"
	cargo run
