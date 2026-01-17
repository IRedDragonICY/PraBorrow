use flutter_rust_bridge::frb;
use praborrow_core::{Sovereign, SovereignState, RepatriationToken};

#[frb(opaque)]
pub struct DartSovereignString {
    inner: Sovereign<String>,
}

impl DartSovereignString {
    #[frb(sync)]
    pub fn new(value: String) -> Self {
        Self {
            inner: Sovereign::new(value),
        }
    }

    #[frb(sync)]
    pub fn is_exiled(&self) -> bool {
        self.inner.is_exiled()
    }

    #[frb(sync)]
    pub fn get_value(&self) -> String {
        if self.inner.is_exiled() {
            return "<Exiled>".to_string();
        }
        match self.inner.try_get() {
            Ok(val) => val.clone(),
            Err(_) => "<Exiled>".to_string(),
        }
    }

    pub fn annex(&self) -> anyhow::Result<()> {
        self.inner.annex().map_err(|e| anyhow::anyhow!("Annex error: {:?}", e))
    }
}
