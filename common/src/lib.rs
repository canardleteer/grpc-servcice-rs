//! Obviously, someone would want more than just "setting up logging" in their
//! uniform service utilities.

use tracing::warn;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, Layer, Registry};

/// In general, this should lead to a more common definition, that is uniform for
/// your services fleet, wiring up to your observability stack as
/// appropriate.
///
/// This is somewhat overkill for this example, but get's things in place
/// for the layered approach for tracing.
pub fn setup_logging() {
    // Filter our emissions, based on environment.
    let text_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let text_filter_level = text_filter.max_level_hint();

    // We only intend to ship logs via stdout, in this example.
    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(text_filter);

    // Make a telemetry Subscriber, from the overall Tracing system.
    let subscriber = Registry::default().with(stdout_layer);

    // And set this Subscriber, as the global defaul for this application.
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => {
            warn!("Text to stdout Level set to: {:?}", text_filter_level);
        }
        Err(e) => {
            panic!("Unable to setup logging, failing: {}", e)
        }
    }
}
