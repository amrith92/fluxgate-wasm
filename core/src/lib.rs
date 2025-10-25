mod config;
mod error;
mod gcra;
mod key_builder;
mod limiter;
mod metrics;
mod policy;
mod time;

pub use config::{CheckRequest, CheckResult, FluxgateInit, FluxgatePolicy};
pub use error::{FluxgateError, Result};
pub use limiter::Fluxgate;

use wasm_bindgen::prelude::*;

type JsResult<T> = std::result::Result<T, JsValue>;

#[wasm_bindgen]
pub struct WasmFluxgate {
    inner: Fluxgate,
}

#[wasm_bindgen]
impl WasmFluxgate {
    #[wasm_bindgen(constructor)]
    pub fn new(init_json: String) -> JsResult<WasmFluxgate> {
        let init: FluxgateInit = serde_json::from_str(&init_json)
            .map_err(|err| JsValue::from_str(&format!("init parse error: {err}")))?;
        Fluxgate::new(init)
            .map(|inner| WasmFluxgate { inner })
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[wasm_bindgen]
    pub fn check(&mut self, req_json: String) -> JsResult<String> {
        let req: CheckRequest = serde_json::from_str(&req_json)
            .map_err(|err| JsValue::from_str(&format!("request parse error: {err}")))?;
        let decision = self.inner.check(req);
        serde_json::to_string(&decision)
            .map_err(|err| JsValue::from_str(&format!("result serialize error: {err}")))
    }

    #[wasm_bindgen]
    pub fn check_batch(&mut self, reqs_json: String) -> JsResult<String> {
        let reqs: Vec<CheckRequest> = serde_json::from_str(&reqs_json)
            .map_err(|err| JsValue::from_str(&format!("batch parse error: {err}")))?;
        let decisions = self.inner.check_batch(reqs);
        serde_json::to_string(&decisions)
            .map_err(|err| JsValue::from_str(&format!("batch serialize error: {err}")))
    }

    #[wasm_bindgen]
    pub fn rotate(&mut self) {
        self.inner.rotate();
    }

    #[wasm_bindgen]
    pub fn reload(&mut self, init_json: String) -> JsResult<()> {
        let init: FluxgateInit = serde_json::from_str(&init_json)
            .map_err(|err| JsValue::from_str(&format!("reload parse error: {err}")))?;
        self.inner
            .reload(init)
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[wasm_bindgen]
    pub fn snapshot(&self) -> JsResult<Vec<u8>> {
        self.inner
            .snapshot()
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[wasm_bindgen]
    pub fn restore(&mut self, bytes: &[u8]) -> JsResult<()> {
        self.inner
            .restore(bytes)
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[wasm_bindgen]
    pub fn metrics(&self) -> JsResult<String> {
        let metrics = self.inner.metrics();
        serde_json::to_string(&metrics)
            .map_err(|err| JsValue::from_str(&format!("metrics serialize error: {err}")))
    }

    #[wasm_bindgen]
    pub fn version(&self) -> String {
        self.inner.version()
    }
}
