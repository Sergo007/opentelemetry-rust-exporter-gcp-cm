mod data_point_to_time_series;
mod histogram_data_point_to_time_series;
mod to_f64;
mod utils;

use itertools::Itertools;
use opentelemetry_resourcedetector_gcp_rust::mapping::get_monitored_resource;

use opentelemetry_sdk::{
    error::OTelSdkError,
    metrics::{
        Temporality,
        data::{AggregatedMetrics, Metric as OpentelemetrySdkMetric, MetricData, ResourceMetrics},
        exporter::PushMetricExporter as PushMetricsExporter,
    },
};

use rand::Rng;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    sync::Arc,
    time::{Duration, SystemTime},
};
#[cfg(feature = "tokio")]
use tokio::sync::RwLock;

use utils::{get_data_points_attributes_keys, normalize_label_key};

use crate::exporter::utils::get_project_id;

pub(crate) const UNIQUE_IDENTIFIER_KEY: &str = "opentelemetry_id";

/// Implementation of Metrics Exporter to Google Cloud Monitoring.
pub struct GCPMetricsExporter {
    prefix: String,
    project_id: String,
    add_unique_identifier: bool,
    unique_identifier: String,
    metric_service: google_cloud_monitoring_v3::client::MetricService,
    metric_descriptors: Arc<RwLock<HashMap<String, google_cloud_api::model::MetricDescriptor>>>,
    custom_monitored_resource_data: Option<MonitoredResourceDataConfig>,
}

/// Configuration for the GCP metrics exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GCPMetricsExporterConfig {
    /// prefix: the prefix of the metric. It is "workload.googleapis.com" by
    ///     default if not specified.
    pub prefix: String,
    /// project id of your Google Cloud project. It is get from GcpAuthorizer by default.
    pub project_id: Option<String>,
    /// add_unique_identifier: Add an identifier to each exporter metric. This
    ///     must be used when there exist two (or more) exporters that may
    ///     export to the same metric name within WRITE_INTERVAL seconds of
    ///     each other.
    pub add_unique_identifier: bool,
    /// custom_monitored_resource_data: Custom monitored resource data to be
    pub custom_monitored_resource_data: Option<MonitoredResourceDataConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Custom monitored resource data
/// need to resolve error 'INVALID_ARGUMENT: One or more TimeSeries could not be written'
/// if we use it we ignore our gcp resource detector and use this data for creating monitored resource
/// https://cloud.google.com/monitoring/api/resources#tag_global
pub struct MonitoredResourceDataConfig {
    pub r#type: String,
    pub labels: HashMap<String, String>,
}

impl Default for GCPMetricsExporterConfig {
    fn default() -> Self {
        Self {
            prefix: "workload.googleapis.com".to_string(),
            project_id: None,
            add_unique_identifier: false,
            custom_monitored_resource_data: None,
        }
    }
}

impl GCPMetricsExporter {
    pub(crate) fn new(
        metric_service: google_cloud_monitoring_v3::client::MetricService,
        project_id: String,
        config: GCPMetricsExporterConfig,
    ) -> Self {
        let my_rundom = format!("{:08x}", rand::rng().random_range(0..u32::MAX));
        Self {
            prefix: config.prefix,
            add_unique_identifier: config.add_unique_identifier,
            project_id: project_id,
            unique_identifier: my_rundom,
            metric_service,
            metric_descriptors: Arc::new(RwLock::new(HashMap::new())),
            custom_monitored_resource_data: config.custom_monitored_resource_data,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GCPMetricsExporterInitError {
    #[error("could not init gcp credentials")]
    InitCredentials(#[source] google_cloud_gax::client_builder::Error),
    #[error("could not detect project id automatically")]
    ProjectIdDedection(#[source] std::io::Error),
}

impl GCPMetricsExporter {
    pub async fn init(config: GCPMetricsExporterConfig) -> Result<GCPMetricsExporter, GCPMetricsExporterInitError> {
        let client = google_cloud_monitoring_v3::client::MetricService::builder()
            .build()
            .await
            .map_err(GCPMetricsExporterInitError::InitCredentials)?;

        let custom_monitored_resource_data_project_id = config
            .custom_monitored_resource_data
            .as_ref()
            .and_then(|v| v.labels.get("project_id").cloned());

        let project_id = config.project_id.clone().or(custom_monitored_resource_data_project_id);

        let project_id = match project_id {
            Some(project_id) => project_id,
            None => match get_project_id().await {
                Ok(proj) => proj,
                Err(err) => {
                    return Err(GCPMetricsExporterInitError::ProjectIdDedection(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        err,
                    )));
                }
            },
        };

        Ok(GCPMetricsExporter::new(client, project_id, config))
    }
}

impl Debug for GCPMetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Google monitoring metrics exporter")
    }
}

impl GCPMetricsExporter {
    /// We can map Metric to MetricDescriptor using Metric.name or
    /// MetricDescriptor.type. We create the MetricDescriptor if it doesn't
    /// exist already and cache it. Note that recreating MetricDescriptors is
    /// a no-op if it already exists.
    ///
    /// :param record:
    /// :return:
    async fn get_metric_descriptor(
        &self,
        metric: &OpentelemetrySdkMetric,
    ) -> Option<google_cloud_api::model::MetricDescriptor> {
        let descriptor_type = format!("{}/{}", self.prefix, metric.name());
        let cached_metric_descriptor = {
            let metric_descriptors = self.metric_descriptors.read().await;
            metric_descriptors.get(&descriptor_type).cloned()
        };
        if let Some(cached_metric_descriptor) = cached_metric_descriptor {
            return Some(cached_metric_descriptor);
        }

        let unit = metric.unit().to_string();
        let mut descriptor = google_cloud_api::model::MetricDescriptor::new()
            .set_type(descriptor_type.clone())
            .set_display_name(metric.name().to_string())
            .set_description(metric.description().to_string())
            .set_unit(unit);

        let seen_keys: HashSet<String> = get_data_points_attributes_keys(metric.data());

        for key in &seen_keys {
            descriptor
                .labels
                .push(google_cloud_api::model::LabelDescriptor::new().set_key(normalize_label_key(key)));
        }

        // todo add unique identifier
        if self.add_unique_identifier {
            descriptor
                .labels
                .push(google_cloud_api::model::LabelDescriptor::new().set_key(UNIQUE_IDENTIFIER_KEY.to_string()));
        }

        match metric.data() {
            AggregatedMetrics::F64(v) => match v {
                MetricData::Histogram(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Cumulative;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Distribution;
                }
                MetricData::ExponentialHistogram(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Cumulative;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Distribution;
                }
                MetricData::Sum(m) => {
                    descriptor.metric_kind = if m.is_monotonic() {
                        google_cloud_api::model::metric_descriptor::MetricKind::Cumulative
                    } else {
                        google_cloud_api::model::metric_descriptor::MetricKind::Gauge
                    };
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Double;
                }
                MetricData::Gauge(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Gauge;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Double;
                }
            },
            AggregatedMetrics::I64(v) => match v {
                MetricData::Histogram(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Cumulative;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Distribution;
                }
                MetricData::ExponentialHistogram(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Cumulative;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Distribution;
                }
                MetricData::Sum(m) => {
                    descriptor.metric_kind = if m.is_monotonic() {
                        google_cloud_api::model::metric_descriptor::MetricKind::Cumulative
                    } else {
                        google_cloud_api::model::metric_descriptor::MetricKind::Gauge
                    };
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Int64;
                }
                MetricData::Gauge(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Gauge;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Int64;
                }
            },
            AggregatedMetrics::U64(v) => match v {
                MetricData::Histogram(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Cumulative;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Distribution;
                }
                MetricData::ExponentialHistogram(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Cumulative;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Distribution;
                }
                MetricData::Sum(m) => {
                    descriptor.metric_kind = if m.is_monotonic() {
                        google_cloud_api::model::metric_descriptor::MetricKind::Cumulative
                    } else {
                        google_cloud_api::model::metric_descriptor::MetricKind::Gauge
                    };
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Int64;
                }
                MetricData::Gauge(_) => {
                    descriptor.metric_kind = google_cloud_api::model::metric_descriptor::MetricKind::Gauge;
                    descriptor.value_type = google_cloud_api::model::metric_descriptor::ValueType::Int64;
                }
            },
        }

        let req = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name(format!("projects/{}", self.project_id.clone()))
            .set_metric_descriptor(descriptor.clone());

        match self
            .metric_service
            .create_metric_descriptor()
            .with_request(req)
            .send()
            .await
        {
            Ok(_) => {}
            Err(err) => {
                match err.status() {
                    Some(status) if status.code == google_cloud_gax::error::rpc::Code::AlreadyExists => {
                        // Metric descriptor already exists, this is fine.
                        let mut metric_descriptors = self.metric_descriptors.write().await;
                        metric_descriptors.insert(descriptor_type, descriptor.clone());
                        return Some(descriptor);
                    }
                    Some(status) if status.code == google_cloud_gax::error::rpc::Code::PermissionDenied => {
                        // Metric descriptor already exists, this is fine.
                        tracing::warn!(
                            "GCPMetricsExporter: PermissionDenied need access with role: `Monitoring Metric Writer` or permissions: `monitoring.metricDescriptors.create`, `monitoring.timeSeries.create`"
                        );
                        return None;
                    }
                    _ => {
                        // Other errors are logged and we return None.
                    }
                }
                tracing::debug!("GCPMetricsExporter: Cant create metric descriptor: {:?}", err);
                return None;
            }
        }

        {
            let mut metric_descriptors = self.metric_descriptors.write().await;
            metric_descriptors.insert(descriptor_type, descriptor.clone());
        }
        Some(descriptor)
    }

    async fn exec_export(&self, metrics: &ResourceMetrics) -> Result<(), OTelSdkError> {
        // // println!("export: {:#?}", metrics);
        // let proto_message: ExportMetricsServiceRequest = (&*metrics).into();
        // // println!("export: {}", serde_json::to_string_pretty(&proto_message).unwrap());

        // use std::io::Write;
        // let mut file = std::fs::File::create("metrics.txt").unwrap();
        // file.write_all(format!("{:#?}", metrics).as_bytes()).unwrap();
        // let monitored_resource_data = match self.custom_monitored_resource_data.clone() {
        //     Some(custom_monitored_resource_data) => Some(google_cloud_api::model::MonitoredResource {
        //         r#type: custom_monitored_resource_data.r#type,
        //         labels: custom_monitored_resource_data.labels,
        //     }),
        //     None => get_monitored_resource(metrics.resource()).map(|v| google_cloud_api::model::MonitoredResource {
        //         r#type: v.r#type,
        //         labels: v.labels,
        //     }),
        // };
        let monitored_resource_data = match self.custom_monitored_resource_data.clone() {
            Some(custom_monitored_resource_data) => Some(
                google_cloud_api::model::MonitoredResource::new()
                    .set_type(custom_monitored_resource_data.r#type)
                    .set_labels(custom_monitored_resource_data.labels),
            ),
            None => get_monitored_resource(metrics.resource()).map(|v| {
                google_cloud_api::model::MonitoredResource::new()
                    .set_type(v.r#type)
                    .set_labels(v.labels)
            }),
        };

        let mut all_series = Vec::<google_cloud_monitoring_v3::model::TimeSeries>::new();
        for scope_metric in metrics.scope_metrics() {
            for metric in scope_metric.metrics() {
                let descriptor = if let Some(descriptor) = self.get_metric_descriptor(metric).await {
                    descriptor
                } else {
                    continue;
                };
                match metric.data() {
                    AggregatedMetrics::F64(v) => match v {
                        MetricData::Histogram(m) => {
                            for data_point in m.data_points() {
                                all_series.push(histogram_data_point_to_time_series::convert(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.as_str(),
                                ));
                            }
                        }
                        MetricData::ExponentialHistogram(m) => {
                            for data_point in m.data_points() {
                                all_series.push(histogram_data_point_to_time_series::convert_exponential(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.as_str(),
                                ));
                            }
                        }
                        MetricData::Sum(m) => {
                            for data_point in m.data_points() {
                                all_series.push(data_point_to_time_series::sum_convert_f64(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.clone(),
                                ));
                            }
                        }
                        MetricData::Gauge(m) => {
                            for data_point in m.data_points() {
                                all_series.push(data_point_to_time_series::gauge_convert_f64(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.clone(),
                                ));
                            }
                        }
                    },
                    AggregatedMetrics::I64(v) => match v {
                        MetricData::Histogram(m) => {
                            for data_point in m.data_points() {
                                all_series.push(histogram_data_point_to_time_series::convert(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.as_str(),
                                ));
                            }
                        }
                        MetricData::ExponentialHistogram(m) => {
                            for data_point in m.data_points() {
                                all_series.push(histogram_data_point_to_time_series::convert_exponential(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.as_str(),
                                ));
                            }
                        }
                        MetricData::Sum(m) => {
                            for data_point in m.data_points() {
                                all_series.push(data_point_to_time_series::sum_convert_i64(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.clone(),
                                ));
                            }
                        }
                        MetricData::Gauge(m) => {
                            for data_point in m.data_points() {
                                all_series.push(data_point_to_time_series::gauge_convert_i64(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.clone(),
                                ));
                            }
                        }
                    },
                    AggregatedMetrics::U64(v) => match v {
                        MetricData::Histogram(m) => {
                            for data_point in m.data_points() {
                                all_series.push(histogram_data_point_to_time_series::convert(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.as_str(),
                                ));
                            }
                        }
                        MetricData::ExponentialHistogram(m) => {
                            for data_point in m.data_points() {
                                all_series.push(histogram_data_point_to_time_series::convert_exponential(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.as_str(),
                                ));
                            }
                        }
                        MetricData::Sum(m) => {
                            for data_point in m.data_points() {
                                all_series.push(data_point_to_time_series::sum_convert_i64(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.clone(),
                                ));
                            }
                        }
                        MetricData::Gauge(m) => {
                            for data_point in m.data_points() {
                                all_series.push(data_point_to_time_series::gauge_convert_i64(
                                    data_point,
                                    &m.start_time(),
                                    &m.time(),
                                    &descriptor,
                                    &monitored_resource_data,
                                    self.add_unique_identifier,
                                    self.unique_identifier.clone(),
                                ));
                            }
                        }
                    },
                }
            }
        }
        // println!("all_series len: {}", all_series.len());
        let chunked_all_series: Vec<Vec<google_cloud_monitoring_v3::model::TimeSeries>> = all_series
            .into_iter()
            .chunks(200)
            .into_iter()
            .map(|chunk| chunk.collect())
            .collect();
        // todo add more usefull error handling and retry
        let project_id = self.project_id.clone();
        for chunk in chunked_all_series {
            let req = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
                .set_name(format!("projects/{}", project_id))
                .set_time_series(chunk.clone());

            match self.metric_service.create_time_series().with_request(req).send().await {
                Ok(_) => {}
                Err(err) => {
                    match err.status() {
                        Some(status) if status.code == google_cloud_gax::error::rpc::Code::PermissionDenied => {
                            tracing::warn!(
                                "GCPMetricsExporter: PermissionDenied need access with role: `Monitoring Metric Writer` or permissions: `monitoring.metricDescriptors.create`, `monitoring.timeSeries.create`"
                            );
                            break;
                        }
                        _ => {}
                    }
                    tracing::debug!("GCPMetricsExporter: Cant send time series: {:?}", err);
                    continue;
                }
            }
        }
        Ok(())
    }
}

impl PushMetricsExporter for GCPMetricsExporter {
    fn export(&self, metrics: &ResourceMetrics) -> impl std::future::Future<Output = Result<(), OTelSdkError>> + Send {
        async {
            let sys_time = SystemTime::now();
            let resp = self.exec_export(metrics).await;
            let new_sys_time = SystemTime::now();
            let _difference = new_sys_time
                .duration_since(sys_time)
                .expect("Clock may have gone backwards")
                .as_millis();
            // info!("export time: {}", difference);
            resp
        }
    }

    fn force_flush(&self) -> Result<(), OTelSdkError> {
        Ok(()) // In this implementation, flush does nothing
    }

    fn temporality(&self) -> Temporality {
        Temporality::default()
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }
}
