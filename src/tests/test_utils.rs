use opentelemetry_sdk;
use std::collections::HashMap;
use std::sync::Arc;

use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{SdkMeterProvider, periodic_reader_with_async_runtime::PeriodicReader};
use tokio::sync::RwLock;

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct GcmCall {
    pub message: String,
}

#[cfg(test)]
pub(crate) type GcmCalls = Arc<RwLock<HashMap<String, Vec<GcmCall>>>>;

#[cfg(test)]
#[derive(Default, Debug, Clone)]
pub(crate) struct MockMetricService {
    pub calls: GcmCalls,
}

#[cfg(test)]
pub async fn unimplemented_stub<T: Send>() -> google_cloud_gax::Result<google_cloud_gax::response::Response<T>> {
    unimplemented!("Mock method not implemented");
}

#[cfg(test)]
impl MockMetricService {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn expect_create_metric_descriptor(
        &self,
    ) -> Vec<google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest> {
        let res = self.calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                serde_json::from_str::<google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest>(&v.message)
                    .unwrap()
            })
            .collect();
        create_metric_descriptor
    }

    pub async fn expect_create_time_series(&self) -> Vec<google_cloud_monitoring_v3::model::CreateTimeSeriesRequest> {
        let res = self.calls.read().await;
        let create_metric_descriptor = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                serde_json::from_str::<google_cloud_monitoring_v3::model::CreateTimeSeriesRequest>(&v.message).unwrap()
            })
            .collect();
        create_metric_descriptor
    }
}

#[cfg(test)]
impl google_cloud_monitoring_v3::stub::MetricService for MockMetricService {
    /// Implements [super::client::MetricService::list_monitored_resource_descriptors].
    fn list_monitored_resource_descriptors(
        &self,
        _req: google_cloud_monitoring_v3::model::ListMonitoredResourceDescriptorsRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<
        Output = google_cloud_monitoring_v3::Result<
            google_cloud_gax::response::Response<
                google_cloud_monitoring_v3::model::ListMonitoredResourceDescriptorsResponse,
            >,
        >,
    > + Send {
        unimplemented_stub()
    }

    /// Implements [super::client::MetricService::get_monitored_resource_descriptor].
    fn get_monitored_resource_descriptor(
        &self,
        _req: google_cloud_monitoring_v3::model::GetMonitoredResourceDescriptorRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<
        Output = google_cloud_monitoring_v3::Result<
            google_cloud_gax::response::Response<google_cloud_api::model::MonitoredResourceDescriptor>,
        >,
    > + Send {
        unimplemented_stub()
    }

    /// Implements [super::client::MetricService::list_metric_descriptors].
    fn list_metric_descriptors(
        &self,
        _req: google_cloud_monitoring_v3::model::ListMetricDescriptorsRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<
        Output = google_cloud_monitoring_v3::Result<
            google_cloud_gax::response::Response<google_cloud_monitoring_v3::model::ListMetricDescriptorsResponse>,
        >,
    > + Send {
        unimplemented_stub()
    }

    /// Implements [super::client::MetricService::get_metric_descriptor].
    fn get_metric_descriptor(
        &self,
        _req: google_cloud_monitoring_v3::model::GetMetricDescriptorRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<
        Output = google_cloud_monitoring_v3::Result<
            google_cloud_gax::response::Response<google_cloud_api::model::MetricDescriptor>,
        >,
    > + Send {
        unimplemented_stub()
    }

    /// Implements [super::client::MetricService::create_metric_descriptor].
    fn create_metric_descriptor(
        &self,
        req: google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<
        Output = google_cloud_monitoring_v3::Result<
            google_cloud_gax::response::Response<google_cloud_api::model::MetricDescriptor>,
        >,
    > + Send {
        let call = GcmCall {
            message: serde_json::to_string(&req).unwrap(),
        };
        let calls = self.calls.clone();
        Box::pin(async move {
            calls
                .write()
                .await
                .entry("CreateMetricDescriptor".to_string())
                .or_default()
                .push(call);
            Ok(google_cloud_gax::response::Response::from(
                req.metric_descriptor.unwrap(),
            ))
        })
    }

    /// Implements [super::client::MetricService::delete_metric_descriptor].
    fn delete_metric_descriptor(
        &self,
        _req: google_cloud_monitoring_v3::model::DeleteMetricDescriptorRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<Output = google_cloud_monitoring_v3::Result<google_cloud_gax::response::Response<()>>> + Send
    {
        unimplemented_stub()
    }

    /// Implements [super::client::MetricService::list_time_series].
    fn list_time_series(
        &self,
        _req: google_cloud_monitoring_v3::model::ListTimeSeriesRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<
        Output = google_cloud_monitoring_v3::Result<
            google_cloud_gax::response::Response<google_cloud_monitoring_v3::model::ListTimeSeriesResponse>,
        >,
    > + Send {
        unimplemented_stub()
    }

    /// Implements [super::client::MetricService::create_time_series].
    fn create_time_series(
        &self,
        req: google_cloud_monitoring_v3::model::CreateTimeSeriesRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<Output = google_cloud_monitoring_v3::Result<google_cloud_gax::response::Response<()>>> + Send
    {
        let call = GcmCall {
            message: serde_json::to_string(&req).unwrap(),
        };
        let calls = self.calls.clone();
        Box::pin(async move {
            calls
                .write()
                .await
                .entry("CreateTimeSeries".to_string())
                .or_default()
                .push(call);
            Ok(google_cloud_gax::response::Response::from(()))
        })
    }

    /// Implements [super::client::MetricService::create_service_time_series].
    fn create_service_time_series(
        &self,
        _req: google_cloud_monitoring_v3::model::CreateTimeSeriesRequest,
        _options: google_cloud_gax::options::RequestOptions,
    ) -> impl std::future::Future<Output = google_cloud_monitoring_v3::Result<google_cloud_gax::response::Response<()>>> + Send
    {
        unimplemented_stub()
    }
}

#[cfg(test)]
pub(crate) fn init_metrics_exporter<T: google_cloud_monitoring_v3::stub::MetricService + 'static>(
    mock_service: T,
) -> crate::GCPMetricsExporter {
    let client = google_cloud_monitoring_v3::client::MetricService::from_stub(mock_service);
    let exporter = crate::GCPMetricsExporter::new(
        client,
        "fake_project_id".to_string(),
        crate::GCPMetricsExporterConfig::default(),
    );
    exporter
}

#[cfg(test)]
pub(crate) fn init_metrics<T: google_cloud_monitoring_v3::stub::MetricService + 'static>(
    mock_service: T,
    res_attributes: Vec<opentelemetry::KeyValue>,
) -> SdkMeterProvider {
    let exporter = init_metrics_exporter(mock_service);
    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();
    SdkMeterProvider::builder()
        .with_resource(
            Resource::builder_empty()
                .with_attributes(res_attributes.clone())
                .build(),
        )
        .with_reader(reader)
        .build()
}
