use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Transport types for OTLP export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryTransport {
    Grpc,
    Http,
}

/// Configuration for the telemetry subsystem.
#[derive(Debug)]
pub struct TelemetryConfig {
    service_name: String,
    otlp_endpoint: Option<String>,
    log_level: String,
    transport: TelemetryTransport,
}

impl TelemetryConfig {
    /// Creates a new configuration builder with default settings.
    pub fn builder() -> TelemetryConfigBuilder {
        TelemetryConfigBuilder::default()
    }

    /// Initializes the telemetry subsystem with this configuration.
    pub fn init(self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // Set global propagator
        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

        // Create OTLP exporter
        // Note: Currently we only support gRPC (tonic) as per original implementation
        // but the config allows for future HTTP support.
        let mut exporter = opentelemetry_otlp::new_exporter().tonic();

        if let Some(endpoint) = self.otlp_endpoint {
            exporter = exporter.with_endpoint(endpoint);
        }

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
                opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                    "service.name",
                    self.service_name,
                )]),
            ))
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        // Create tracing layer
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        // Create env filter layer
        let filter = tracing_subscriber::EnvFilter::new(self.log_level);

        // Initialize subscriber
        tracing_subscriber::registry()
            .with(telemetry)
            .with(filter)
            .try_init()?;

        Ok(())
    }
}

/// Builder for `TelemetryConfig`.
#[derive(Default)]
pub struct TelemetryConfigBuilder {
    service_name: Option<String>,
    otlp_endpoint: Option<String>,
    log_level: Option<String>,
    transport: Option<TelemetryTransport>,
}

impl TelemetryConfigBuilder {
    /// Sets the service name (required).
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// Sets the OTLP endpoint URL.
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.otlp_endpoint = Some(endpoint.into());
        self
    }

    /// Sets the log level (default: "info").
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = Some(level.into());
        self
    }

    /// Sets the transport protocol (default: Grpc).
    pub fn transport(mut self, transport: TelemetryTransport) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> TelemetryConfig {
        TelemetryConfig {
            service_name: self
                .service_name
                .unwrap_or_else(|| "praborrow-unknown".to_string()),
            otlp_endpoint: self.otlp_endpoint,
            log_level: self.log_level.unwrap_or_else(|| "info".to_string()),
            transport: self.transport.unwrap_or(TelemetryTransport::Grpc),
        }
    }
}

/// Initializes the telemetry subsystem.
///
/// # Deprecated
/// Use `TelemetryConfig::builder().service_name(name).init()` instead.
#[deprecated(since = "0.8.0", note = "Use TelemetryConfig::builder() instead")]
pub fn init_tracing(
    service_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    TelemetryConfig::builder()
        .service_name(service_name)
        .build()
        .init()
}

/// Shuts down the telemetry subsystem, flushing pending spans.
pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}
