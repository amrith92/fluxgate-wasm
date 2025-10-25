const path = require('path');
const { createRequire } = require('module');

const requireFromHere = createRequire(__filename);

let wasmInitPromise = null;

async function ensureWasm() {
  if (!wasmInitPromise) {
    const init = requireFromHere('../pkg/fluxgate_wasm.js');
    if (typeof init === 'function') {
      wasmInitPromise = init();
    } else {
      wasmInitPromise = Promise.resolve();
    }
  }
  await wasmInitPromise;
}

exports.createFluxgate = async function createFluxgate(init) {
  await ensureWasm();
  const { pathToFileURL } = require('url');
  const fileUrl = pathToFileURL(path.join(__dirname, 'index.js')).href;
  const esm = await import(fileUrl);
  return esm.createFluxgate(init);
};
