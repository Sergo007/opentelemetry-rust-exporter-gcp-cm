#[cfg(test)]
mockall::mock! {
    #[derive(Debug)]
    pub MetricService {}

    impl google_cloud_monitoring_v3::stub::MetricService for MetricService {
        fn create_metric_descriptor(
            &self,
            _req: google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest,
            _options: google_cloud_gax::options::RequestOptions,
        ) -> impl std::future::Future<
            Output = google_cloud_monitoring_v3::Result<google_cloud_gax::response::Response<google_cloud_api::model::MetricDescriptor>>,
        > + Send;
    }
}

async fn try_monitoring_sdk(metric_service: &google_cloud_monitoring_v3::client::MetricService) {
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

    let resp = metric_service
        .create_metric_descriptor()
        .with_request(req)
        .send()
        .await
        .unwrap();

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
        let mut mock_service = MockMetricService::new();

        // Setup mock expectations
        mock_service
            .expect_create_metric_descriptor()
            .times(1)
            .withf(move |req, _options| {
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
            })
            .returning(|_req, _options| {
                let resp = google_cloud_api::model::MetricDescriptor::new()
                    .set_type("custom.googleapis.com/opencensus/cybx.io/try_monitoring_sdk")
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                    .set_unit("1")
                    .set_description("My custom metric")
                    .set_display_name("My Metric");
                let response = google_cloud_gax::response::Response::from(resp);
                Box::pin(async move { Ok(response) })
            });

        // Call the function with the mock

        let client = google_cloud_monitoring_v3::client::MetricService::from_stub(mock_service);

        try_monitoring_sdk(&client).await;

        // Mock will automatically verify that create_metric_descriptor_call was called exactly once
        // with the expected parameters when it goes out of scope
    }
}
