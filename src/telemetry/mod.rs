use tokio::task::JoinHandle;
use tracing::{Subscriber, dispatcher::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

pub fn get_tracing_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");

    set_global_default(subscriber.into()).expect("Failed to set subscriber");
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();

    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}

pub trait TraceError<T, E> {
    fn trace_with(self, context: &str) -> Result<T, E>;
}

impl<T, E> TraceError<T, E> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn trace_with(self, context: &str) -> Result<T, E> {
        self.map_err(|e| {
            let span = tracing::Span::current();
            span.in_scope(|| {
                tracing::error!(target: "thalia_errors" ,error=%e, error_debug=?e, context=context);
            });
            e
        })
    }
}
