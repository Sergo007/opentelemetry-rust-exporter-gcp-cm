#[cfg_attr(test, mockall::automock)]
trait MetricServiceTrait {
    async fn create_metric_descriptor_call(
        &self,
        req: google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest,
    ) -> Result<google_cloud_api::model::MetricDescriptor, Box<dyn std::error::Error>>;
}

impl MetricServiceTrait for google_cloud_monitoring_v3::client::MetricService {
    async fn create_metric_descriptor_call(
        &self,
        req: google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest,
    ) -> Result<google_cloud_api::model::MetricDescriptor, Box<dyn std::error::Error>> {
        let resp = self.create_metric_descriptor().with_request(req).send().await?;
        Ok(resp)
    }
}

async fn try_monitoring_sdk<T: MetricServiceTrait>(metric_service: &T) {
    let req = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
        .set_name("projects/cybx-chat".to_string())
        .set_metric_descriptor(
            google_cloud_api::model::MetricDescriptor::new()
                .set_type("custom.googleapis.com/opencensus/cybx.io/try_monitoring_sdk")
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                .set_unit("1")
                .set_description("My custom metric")
                .set_display_name("My Metric"),
        );
    // println!(
    //     "CreateMetricDescriptorRequest: {}",
    //     serde_json::to_string_pretty(&req).unwrap()
    // );
    // println!("CreateMetricDescriptorRequest: {:?}", req);

    let resp = metric_service.create_metric_descriptor_call(req).await.unwrap();

    println!(
        "Created Metric Descriptor: {}",
        serde_json::to_string_pretty(&resp).unwrap()
    );
}

#[tokio::main]
async fn main() {
    std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/Users/serhiiyatsina/projects/cybx/opentelemetry/opentelemetry-rust-exporter-gcp-cm/.secrets/977645940426-compute@developer.gserviceaccount.com.json");

    let client = google_cloud_monitoring_v3::client::MetricService::builder()
        .build()
        .await
        .unwrap();

    try_monitoring_sdk(&client).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted_fork::assert_eq_all_sorted;

    #[tokio::test]
    async fn test_try_monitoring_sdk_with_mock() {
        let mut mock_service = MockMetricServiceTrait::new();

        // Setup mock expectations
        mock_service
            .expect_create_metric_descriptor_call()
            .times(1)
            .withf(
                move |req: &google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest| {
                    // Verify the request parameters
                    let should_be = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
                        .set_name("projects/cybx-chat".to_string())
                        .set_metric_descriptor(
                            google_cloud_api::model::MetricDescriptor::new()
                                .set_type("custom.googleapis.com/opencensus/cybx.io/try_monitoring_sdk")
                                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                                .set_unit("1")
                                .set_description("My custom metric")
                                .set_display_name("My Metric"),
                        );
                    assert_eq_all_sorted!(&req, &should_be);
                    true
                },
            )
            .returning(|_req| {
                Ok(google_cloud_api::model::MetricDescriptor::new()
                    .set_type("custom.googleapis.com/opencensus/cybx.io/try_monitoring_sdk")
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                    .set_unit("1")
                    .set_description("My custom metric")
                    .set_display_name("My Metric"))
            });

        // Call the function with the mock
        try_monitoring_sdk(&mock_service).await;

        // Mock will automatically verify that create_metric_descriptor_call was called exactly once
        // with the expected parameters when it goes out of scope
    }
}
