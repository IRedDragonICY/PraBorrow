uniffi::setup_scaffolding!("praborrow");

use praborrow_core::{Sovereign, SovereignState};
use std::sync::Arc;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BindingError {
    #[error("Invalid input")]
    InvalidInput,
    #[error("Sovereignty violation")]
    SovereigntyViolation,
    #[error("Annexation error")]
    AnnexationError,
}

#[derive(uniffi::Object)]
pub struct SovereignString {
    inner: Sovereign<String>,
}

#[uniffi::export]
impl SovereignString {
    #[uniffi::constructor]
    pub fn new(value: String) -> Self {
        Self {
            inner: Sovereign::new(value),
        }
    }

    pub fn get_value(&self) -> String {
        // Safe copy for string
        if self.inner.is_exiled() {
            return "<Exiled>".to_string();
        }
        match self.inner.try_get() {
            Ok(val) => val.clone(),
            Err(_) => "<Exiled>".to_string(),
        }
    }

    pub fn annex(&self) -> Result<(), BindingError> {
        self.inner.annex().map_err(|_| BindingError::AnnexationError)
    }

    pub fn is_exiled(&self) -> bool {
        self.inner.is_exiled()
    }

    pub fn is_domestic(&self) -> bool {
        self.inner.is_domestic()
    }
}
