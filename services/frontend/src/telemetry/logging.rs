use std::io;

use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// Compose multiple layers into a tracing subscriber.
///
/// The layers used allow filtering tracing spans based on the logging level set as an environment
/// variable. The logs are sent to stdout in Bunyan compatible format.
pub fn make_subscriber(name: impl Into<String>, env_filter: impl Into<String>) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter.into()));
    let formatting_layer = BunyanFormattingLayer::new(name.into(), io::stdout);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Initializes the given tracing subscriber by setting it as global default. This function also
/// redirects all `log` calls to the given subscriber.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");

    set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
