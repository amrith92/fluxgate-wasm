import initWasm, * as wasm from '../pkg/fluxgate_wasm.js';
import type {
  Fluxgate,
  FluxgateInit,
  CheckRequest,
  CheckResult,
} from './types.js';

let wasmReady: Promise<unknown> | null = null;

async function ensureWasmLoaded(): Promise<void> {
  if (!wasmReady) {
    wasmReady = initWasm();
  }
  await wasmReady;
}

function parseResult(result: string): CheckResult {
  return JSON.parse(result) as CheckResult;
}

export async function createFluxgate(init: FluxgateInit): Promise<Fluxgate> {
  await ensureWasmLoaded();
  const ctor = (wasm as any).WasmFluxgate;
  if (!ctor) {
    throw new Error('WasmFluxgate constructor is not available. Did you run `wasm-pack build`?');
  }
  const instance = new ctor(JSON.stringify(init));

  return {
    check(req: CheckRequest): CheckResult {
      const response = instance.check(JSON.stringify(req));
      return parseResult(response);
    },
    checkBatch(reqs: CheckRequest[]): CheckResult[] {
      const response = instance.check_batch(JSON.stringify(reqs));
      return JSON.parse(response) as CheckResult[];
    },
    rotate(): void {
      instance.rotate();
    },
    reload(cfg: FluxgateInit): void {
      instance.reload(JSON.stringify(cfg));
    },
    snapshot(): Uint8Array {
      const bytes = instance.snapshot();
      return bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
    },
    restore(bytes: Uint8Array): void {
      instance.restore(bytes);
    },
    metrics(): Record<string, number> {
      const response = instance.metrics();
      return JSON.parse(response) as Record<string, number>;
    },
    version(): string {
      return instance.version();
    },
  };
}
