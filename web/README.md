# Slugline — Frontend

Svelte 5 + Vite + TypeScript frontend. Built by `make build` and embedded into the Rust binary via `rust-embed`. See the [root README](../README.md) for full project documentation.

## Development

```sh
# Install dependencies
npm install

# Run Vite dev server (proxies /api to the Rust backend on :4747)
npm run dev

# Production build (output to web/dist/, consumed by cargo build)
npm run build

# Unit tests
npm test

# Type-check
npm run check
```
