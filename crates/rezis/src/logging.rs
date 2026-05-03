//! One-shot [`tracing_subscriber`] setup for [`crate::RezisApp::with_logging`](crate::RezisApp::with_logging).

use std::sync::OnceLock;

use tracing_subscriber::EnvFilter;

static TRACING_INIT: OnceLock<()> = OnceLock::new();

/// Ensures `tracing` is wired to stderr (`fmt`) at most once. Honors `RUST_LOG` when valid;
/// otherwise defaults to **info** level for Rezis and dependencies.
pub(crate) fn ensure_tracing_initialized() {
    TRACING_INIT.get_or_init(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    });
}
