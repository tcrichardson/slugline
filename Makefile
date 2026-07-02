.PHONY: run dev test fmt build dist

# Run the app with the default notes directory (~/Documents/Slugline).
run:
	cargo run -p slugline

# Run with a throwaway notes dir, for local development.
dev:
	cargo run -p slugline -- --notes-dir ./dev-notes

test:
	cargo test --workspace

fmt:
	cargo fmt

# Production build: a single self-contained release binary.
build:
	cargo build --release -p slugline

dist: build
	@echo "Built single binary:"
	@ls -lh target/release/slugline
