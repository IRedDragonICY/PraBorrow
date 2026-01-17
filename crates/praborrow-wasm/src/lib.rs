use wasm_bindgen::prelude::*;
use praborrow_core::{Sovereign, RepatriationToken};

#[wasm_bindgen]
pub fn setup_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct JsSovereign {
    inner: Sovereign<String>,
}

#[wasm_bindgen]
impl JsSovereign {
    #[wasm_bindgen(constructor)]
    pub fn new(resource: String) -> Self {
        Self {
            inner: Sovereign::new(resource),
        }
    }

    pub fn annex(&self) -> Result<(), String> {
        self.inner.annex().map_err(|e| format!("{:?}", e))
    }

    pub fn is_exiled(&self) -> bool {
        self.inner.is_exiled()
    }

    pub fn repatriate(&self, token: JsRepatriationToken) -> Result<(), String> {
        self.inner.repatriate(token.inner);
        Ok(())
    }
}

#[wasm_bindgen]
pub struct JsRepatriationToken {
     inner: RepatriationToken,
}

#[wasm_bindgen]
impl JsRepatriationToken {
    #[wasm_bindgen(constructor)]
    pub fn new(holder_id: u64) -> Self {
        // Safe cast for demo purposes, u128 is not directly supported by wasm-bindgen easily yet without BigInt
        Self {
            inner: unsafe { RepatriationToken::new(holder_id as u128) },
        }
    }
}
