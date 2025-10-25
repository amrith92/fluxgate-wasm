# Fluxgate WASM

This repository packages a lightweight in-memory rate limiter for Node.js by
compiling the Fluxgate core to WebAssembly. The implementation focuses on the
precise tier, exposing a token bucket/GCRA limiter that mirrors the API of the
native project and leaves room for future probabilistic datastructures.

## Layout

- `core/`: Rust source compiled to `wasm32-unknown-unknown` via `wasm-bindgen`.
- `js/`: TypeScript helpers that wrap the raw WASM bindings.
- `examples/`: Small Node.js snippets showing how to load the limiter.

## Building

1. Install Rust 1.77+ and `wasm-pack`.
2. Build the WebAssembly package:

   ```bash
   cd core
   wasm-pack build --target nodejs --release --out-dir ../pkg
   ```

3. Install Node.js dependencies and compile the TypeScript helpers:

   ```bash
   npm install
   npm run build
   ```

## Usage

```ts
import { createFluxgate } from '@fluxgate/wasm';

const gate = await createFluxgate({
  policies: [
    { id: 'ip-global', match: 'ip:*', limitPerSecond: 100, burst: 50, windowSeconds: 60 },
  ],
});

const decision = gate.check({ ip: '203.0.113.8' });
if (!decision.allowed) {
  console.log(`429 retry after ${decision.retryAfterMs}ms`);
}
```

The API also exposes `checkBatch`, `rotate`, `reload`, `snapshot`, `restore`,
`metrics`, and `version` helpers that align with the Fluxgate design document.
