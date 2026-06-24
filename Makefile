.PHONY: dev run test test-web fmt fmt-web build dev-web dist

# Run the backend (serves the embedded SPA + API)
run:
	cargo run

# Backend dev with a throwaway notes dir and no browser auto-open
dev:
	cargo run -- --notes-dir ./dev-notes --no-open

test:
	cargo test

test-web:
	cd web && npm test

fmt:
	cargo fmt

fmt-web:
	cd web && npx prettier --write "src/**/*.{ts,svelte}"

dev-web:
	cd web && npm run dev

# Production build: frontend bundle (Vite default outDir is web/dist) then release binary
build:
	cd web && npm run build
	cargo build --release

dist: build
	@echo "Built single binary:"
	@ls -lh target/release/slugline
