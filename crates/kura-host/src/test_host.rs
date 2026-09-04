//! A minimal [`Host`] for this crate's unit tests.
//!
//! Only [`Host::spawn`] is real: the tests that need a host need it because a
//! relay session must get its socket loop onto the runtime. They already run
//! under `#[tokio::test]`, so a bare `tokio::spawn` is the whole
//! implementation. Every other capability panics — a test that reaches one is
//! exercising something this fake was never meant to stand in for, and a
//! silent dummy would hide that.

use std::path::PathBuf;

use crate::app_state::AppState;
use crate::host::{BoxFuture, Host, HostHandle};

pub(crate) struct TestHost;

/// A [`HostHandle`] whose only working capability is task spawn.
pub(crate) fn test_host() -> HostHandle {
    std::sync::Arc::new(TestHost)
}

impl Host for TestHost {
    fn app_data_dir(&self) -> Result<PathBuf, String> {
        unimplemented!("TestHost has no data directory")
    }

    fn app_config_dir(&self) -> Result<PathBuf, String> {
        unimplemented!("TestHost has no config directory")
    }

    fn state(&self) -> &AppState {
        unimplemented!("TestHost holds no AppState")
    }

    fn emit(&self, _event: &str, _payload: serde_json::Value) -> Result<(), String> {
        unimplemented!("TestHost has no frontend to emit to")
    }

    fn spawn(&self, future: BoxFuture) {
        tokio::spawn(future);
    }

    fn instance_id(&self) -> String {
        "com.kura.test".to_string()
    }

    fn is_dev(&self) -> bool {
        true
    }
}
