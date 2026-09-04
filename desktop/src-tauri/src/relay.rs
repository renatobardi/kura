//! Re-export of [`kura_host::relay`]. The relay client moved into `kura-host`
//! with the rest of the host-agnostic backend; `crate::relay::…` call sites are
//! unchanged.
pub use kura_host::relay::*;
