use std::sync::atomic::AtomicBool;

use once_cell::sync::Lazy;
use tracing_chrome::FlushGuard;

static IS_TRACING_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn enable_tracing_by_env() {
  let is_enable_tracing = std::env::var("TRACE").is_ok();
  if is_enable_tracing {
    if !IS_TRACING_ENABLED.swap(true, std::sync::atomic::Ordering::SeqCst) {
      use tracing_chrome::ChromeLayerBuilder;
      use tracing_subscriber::{fmt, prelude::*, registry::Registry};

      let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
      std::mem::forget(guard);
      tracing_subscriber::registry()
        .with(chrome_layer)
        .with(fmt::layer())
        .init();
    }
  }
}