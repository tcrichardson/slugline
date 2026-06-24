.PHONY: test-web fmt-web

test-web:
	cd web && npm test

fmt-web:
	cd web && npx prettier --write "src/**/*.{ts,svelte}"
