use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes the telemetry subsystem.
///
/// Configures OpenTelemetry export to OTLP (gRPC) and sets up
/// tracing subscribers for structured logging.
///
/// # Environment Variables
///
/// - `OTEL_EXPORTER_OTLP_ENDPOINT`: URL of the OTLP collector (default: <http://localhost:4317>)
/// - `RUST_LOG`: Log level filter (default: info)
///
/// # Errors
///
/// Returns an error if:
/// - The OTLP pipeline cannot be installed (e.g., transport error).
/// - The subscriber initialization fails (e.g., global subscriber already set).
pub fn init_tracing(service_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Set global propagator
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    // Create OTLP exporter
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // Create tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create env filter layer
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Initialize subscriber
    tracing_subscriber::registry()
        .with(telemetry)
        .with(filter)
        .try_init()?;

    Ok(())
}

/// Shuts down the telemetry subsystem, flushing pending spans.
pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}
