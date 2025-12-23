#![allow(unexpected_cfgs)]
mod exporter;

pub use exporter::GCPMetricsExporter;
pub use exporter::GCPMetricsExporterConfig;
pub use exporter::MonitoredResourceDataConfig;

#[cfg(test)]
mod tests;
