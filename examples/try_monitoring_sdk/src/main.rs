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
    println!("CreateMetricDescriptorRequest: {:?}", req);

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
    use pretty_assertions_sorted_fork::assert_eq_all_sorted;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_histogram_default_buckets() {
        let req1 = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
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
        let req2 = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
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

        assert_eq_all_sorted!(&req1, &req2);
    }
}
