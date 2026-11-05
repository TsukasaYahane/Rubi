//! The core and the runtime its async calls run on. Pages reach the core only through this, off
//! the UI thread, and come back with plain data.

use std::future::Future;

use rubi_core::{Services, Session};
use tokio::runtime::Runtime;

pub struct Backend {
    pub services: Services,
    runtime: Runtime,
}

impl Backend {
    pub fn open(data_dir: &str, session: Session) -> Result<Self, String> {
        let services = Services::open(data_dir, session).map_err(|e| e.to_string())?;
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|e| e.to_string())?;
        Ok(Self { services, runtime })
    }

    /// Runs `work` in the background.
    pub fn spawn(&self, work: impl Future<Output = ()> + Send + 'static) {
        self.runtime.spawn(work);
    }
}
