#![allow(unexpected_cfgs)]
#[macro_use]
pub mod error;
mod exporter;
#[cfg(feature = "gcp_auth")]
mod gcp_auth_authorizer;
pub mod gcp_authorizer;
pub mod gcp_authorizer_error;
pub use exporter::GCPMetricsExporter;
pub use exporter::GCPMetricsExporterConfig;
pub use exporter::MonitoredResourceDataConfig;
mod gcloud_sdk;
#[cfg(test)]
mod tests;
