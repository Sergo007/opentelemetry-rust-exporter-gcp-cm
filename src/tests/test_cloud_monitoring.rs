#[cfg(test)]
mod tests {
    use crate::tests::test_utils::*;

    use opentelemetry::metrics::MeterProvider;
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::metrics::{InstrumentKind, StreamBuilder};
    use opentelemetry_sdk::runtime;
    use opentelemetry_sdk::{
        metrics::{periodic_reader_with_async_runtime::PeriodicReader, Aggregation, Instrument, SdkMeterProvider},
        Resource,
    };
    use pretty_assertions_sorted_fork::{assert_eq, assert_eq_all_sorted};
    use std::collections::HashMap;

    fn my_unit() -> String {
        "myunit".to_string()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_histogram_default_buckets() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let histogram = meter
            .f64_histogram("myhistogram")
            .with_description("foo")
            .with_unit(my_unit());

        let histogram = histogram.build();
        for i in 0..10_000 {
            histogram.record(
                i as f64,
                &[
                    KeyValue::new("string", "string"),
                    KeyValue::new("int", 123),
                    KeyValue::new("float", 123.4),
                ],
            );
        }
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myhistogram")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Distribution)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myhistogram")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myhistogram")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Distribution)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_start_time(google_cloud_wkt::Timestamp::new(1723249032, 929246000).unwrap())
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_distribution_value(
                            google_cloud_api::model::Distribution::new()
                                .set_count(10000)
                                .set_mean(4999.5)
                                .set_sum_of_squared_deviation(0.0)
                                .set_bucket_options(
                                    google_cloud_api::model::distribution::BucketOptions::new().set_explicit_buckets(
                                        google_cloud_api::model::distribution::bucket_options::Explicit::new()
                                            .set_bounds(vec![
                                                0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0,
                                                2500.0, 5000.0, 7500.0, 10000.0,
                                            ]),
                                    ),
                                )
                                .set_bucket_counts(vec![
                                    1, 5, 5, 15, 25, 25, 25, 150, 250, 250, 250, 1500, 2500, 2500, 2499, 0,
                                ]),
                        ),
                    )])
                .set_unit("myunit")]);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_histogram_single_bucket() {
        let mock_service = MockMetricService::new();
        let exporter = init_metrics_exporter(mock_service.clone());
        let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
        let my_view_change_aggregation = |i: &Instrument| {
            if i.name() == "my_single_bucket_histogram" {
                if let InstrumentKind::Histogram = i.kind() {
                    let stream = StreamBuilder::default()
                        .with_name(i.name().to_string())
                        // .with_description(i.description().to_string()) // TODO: description() is not supported in opentelemetry_sdk Instrument
                        .with_description("foo".to_string()) // use a static description for testing
                        .with_unit(i.unit().to_string())
                        .with_cardinality_limit(100)
                        .with_aggregation(Aggregation::ExplicitBucketHistogram {
                            boundaries: vec![5.5],
                            record_min_max: true,
                        })
                        .build();
                    return stream.ok();
                }
            }
            None
        };
        let metrics_provider = SdkMeterProvider::builder()
            .with_resource(
                Resource::builder_empty()
                    .with_attributes(vec![KeyValue::new("service.name", "metric-demo")])
                    .build(),
            )
            .with_reader(reader)
            .with_view(my_view_change_aggregation)
            .build();
        // global::set_meter_provider(metrics_provider.clone());

        let meter = metrics_provider.meter("test_cloud_monitoring");
        let histogram = meter
            .f64_histogram("my_single_bucket_histogram")
            .with_description("foo")
            .with_unit(my_unit());

        let histogram = histogram.build();
        for i in 0..10_000 {
            histogram.record(
                i as f64,
                &[
                    KeyValue::new("string", "string"),
                    KeyValue::new("int", 123),
                    KeyValue::new("float", 123.4),
                ],
            );
        }
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/my_single_bucket_histogram")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Distribution)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("my_single_bucket_histogram")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );

        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/my_single_bucket_histogram")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Distribution)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_start_time(google_cloud_wkt::Timestamp::new(1723249032, 929246000).unwrap())
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_distribution_value(
                            google_cloud_api::model::Distribution::new()
                                .set_count(10000)
                                .set_mean(4999.5)
                                .set_sum_of_squared_deviation(0.0)
                                .set_bucket_options(
                                    google_cloud_api::model::distribution::BucketOptions::new().set_explicit_buckets(
                                        google_cloud_api::model::distribution::bucket_options::Explicit::new()
                                            .set_bounds(vec![5.5]),
                                    ),
                                )
                                .set_bucket_counts(vec![6, 9994]),
                        ),
                    )])
                .set_unit("myunit")]);
        // assert_eq_sorted!(create_time_series, expected_create_time_series);

        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_up_down_counter_float() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let updowncounter = meter
            .f64_up_down_counter("myupdowncounter")
            .with_description("foo")
            .with_unit(my_unit());

        let updowncounter = updowncounter.build();
        updowncounter.add(
            45.6,
            &[
                KeyValue::new("string", "string"),
                KeyValue::new("int", 123),
                KeyValue::new("float", 123.4),
            ],
        );
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myupdowncounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myupdowncounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myupdowncounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_double_value(45.6),
                    )])
                .set_unit("myunit")]);
        // assert_eq_sorted!(create_time_series, expected_create_time_series);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_up_down_counter_int() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let updowncounter = meter
            .i64_up_down_counter("myupdowncounter")
            .with_description("foo")
            .with_unit(my_unit());

        let updowncounter = updowncounter.build();
        updowncounter.add(
            45,
            &[
                KeyValue::new("string", "string"),
                KeyValue::new("int", 123),
                KeyValue::new("float", 123.4),
            ],
        );
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myupdowncounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myupdowncounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myupdowncounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_int64_value(45),
                    )])
                .set_unit("myunit")]);
        // assert_eq_sorted!(create_time_series, expected_create_time_series);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_up_down_counter_int() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let updowncounter = meter
            .i64_observable_up_down_counter("myobservablecounter")
            .with_callback(|result| {
                result.observe(
                    45,
                    &[
                        KeyValue::new("string", "string"),
                        KeyValue::new("int", 123),
                        KeyValue::new("float", 123.4),
                    ],
                );
            })
            .with_description("foo")
            .with_unit(my_unit());

        let _updowncounter = updowncounter.build();
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myobservablecounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myobservablecounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myobservablecounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_int64_value(45),
                    )])
                .set_unit("myunit")]);
        // assert_eq_sorted!(create_time_series, expected_create_time_series);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_up_down_counter_float() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let updowncounter = meter
            .f64_observable_up_down_counter("myobservablecounter")
            .with_callback(|result| {
                result.observe(
                    45.0,
                    &[
                        KeyValue::new("string", "string"),
                        KeyValue::new("int", 123),
                        KeyValue::new("float", 123.4),
                    ],
                );
            })
            .with_description("foo")
            .with_unit(my_unit());

        let _updowncounter = updowncounter.build();
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myobservablecounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myobservablecounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myobservablecounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_double_value(45.0),
                    )])
                .set_unit("myunit")]);
        // assert_eq_sorted!(create_time_series, expected_create_time_series);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_counter_int() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let updowncounter = meter
            .u64_observable_counter("myobservablecounter")
            .with_callback(|result| {
                result.observe(
                    45,
                    &[
                        KeyValue::new("string", "string"),
                        KeyValue::new("int", 123),
                        KeyValue::new("float", 123.4),
                    ],
                );
            })
            .with_description("foo")
            .with_unit(my_unit());

        let _updowncounter = updowncounter.build();
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myobservablecounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myobservablecounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myobservablecounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_int64_value(45),
                    )])
                .set_unit("myunit")]);
        // assert_eq_sorted!(create_time_series, expected_create_time_series);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_counter_float() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let updowncounter = meter
            .f64_observable_counter("myobservablecounter")
            .with_callback(|result| {
                result.observe(
                    45.0,
                    &[
                        KeyValue::new("string", "string"),
                        KeyValue::new("int", 123),
                        KeyValue::new("float", 123.4),
                    ],
                );
            })
            .with_description("foo")
            .with_unit(my_unit());

        let _updowncounter = updowncounter.build();
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myobservablecounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myobservablecounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myobservablecounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_double_value(45.0),
                    )])
                .set_unit("myunit")]);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_gauge_int() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let updowncounter = meter
            .u64_observable_gauge("myobservablegauge")
            .with_callback(|result| {
                result.observe(
                    45,
                    &[
                        KeyValue::new("string", "string"),
                        KeyValue::new("int", 123),
                        KeyValue::new("float", 123.4),
                    ],
                );
            })
            .with_description("foo")
            .with_unit(my_unit());

        let _updowncounter = updowncounter.build();
        metrics_provider.force_flush().unwrap();

        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myobservablegauge")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myobservablegauge")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myobservablegauge")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_int64_value(45),
                    )])
                .set_unit("myunit")]);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_gauge_float() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let updowncounter = meter
            .f64_observable_gauge("myobservablegauge")
            .with_callback(|result| {
                result.observe(
                    45.0,
                    &[
                        KeyValue::new("string", "string"),
                        KeyValue::new("int", 123),
                        KeyValue::new("float", 123.4),
                    ],
                );
            })
            .with_description("foo")
            .with_unit(my_unit());

        let _updowncounter = updowncounter.build();
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/myobservablegauge")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("myobservablegauge")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/myobservablegauge")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Gauge)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_double_value(45.0),
                    )])
                .set_unit("myunit")]);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_counter_int() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let mycounter = meter
            .u64_counter("mycounter")
            .with_description("foo")
            .with_unit(my_unit());

        let mycounter = mycounter.build();

        mycounter.add(
            45,
            &[
                KeyValue::new("string", "string"),
                KeyValue::new("int", 123),
                KeyValue::new("float", 123.4),
            ],
        );
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/mycounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("mycounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/mycounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_int64_value(45),
                    )])
                .set_unit("myunit")]);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_counter_float() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let mycounter = meter
            .f64_counter("mycounter")
            .with_description("foo")
            .with_unit(my_unit());

        let mycounter = mycounter.build();

        mycounter.add(
            45.0,
            &[
                KeyValue::new("string", "string"),
                KeyValue::new("int", 123),
                KeyValue::new("float", 123.4),
            ],
        );
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/mycounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("mycounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/mycounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Double)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_double_value(45.0),
                    )])
                .set_unit("myunit")]);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_invalid_label_keys() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(mock_service.clone(), vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let mycounter = meter
            .u64_counter("mycounter")
            .with_description("foo")
            .with_unit(my_unit());

        let mycounter = mycounter.build();

        mycounter.add(12, &[KeyValue::new("1some.invalid$\\key", "value")]);
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/mycounter")
                    .set_labels([google_cloud_api::model::LabelDescriptor::new()
                        .set_key("key_1some_invalid__key")
                        .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String)])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("mycounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/mycounter")
                        .set_labels(HashMap::from([(
                            "key_1some_invalid__key".to_string(),
                            "value".to_string(),
                        )])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("generic_node")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "global".to_string()),
                            ("namespace".to_string(), "".to_string()),
                            ("node_id".to_string(), "".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_int64_value(12),
                    )])
                .set_unit("myunit")]);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_with_resource() {
        let mock_service = MockMetricService::new();
        let metrics_provider = init_metrics(
            mock_service.clone(),
            vec![
                KeyValue::new("cloud.platform", "gcp_kubernetes_engine"),
                KeyValue::new("cloud.availability_zone", "myavailzone"),
                KeyValue::new("k8s.cluster.name", "mycluster"),
                KeyValue::new("k8s.namespace.name", "myns"),
                KeyValue::new("k8s.pod.name", "mypod"),
                KeyValue::new("k8s.container.name", "mycontainer"),
            ],
        );
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let mycounter = meter
            .u64_counter("mycounter")
            .with_description("foo")
            .with_unit(my_unit());
        let mycounter = mycounter.build();

        mycounter.add(
            12,
            &[
                KeyValue::new("string", "string"),
                KeyValue::new("int", 123),
                KeyValue::new("float", 123.4),
            ],
        );
        metrics_provider.force_flush().unwrap();
        let create_metric_descriptor = mock_service.expect_create_metric_descriptor().await;
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = google_cloud_monitoring_v3::model::CreateMetricDescriptorRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_metric_descriptor(
                google_cloud_api::model::MetricDescriptor::new()
                    .set_type("workload.googleapis.com/mycounter")
                    .set_labels([
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("string")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("int")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                        google_cloud_api::model::LabelDescriptor::new()
                            .set_key("float")
                            .set_value_type(google_cloud_api::model::label_descriptor::ValueType::String),
                    ])
                    .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                    .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                    .set_unit("myunit")
                    .set_description("foo")
                    .set_display_name("mycounter")
                    .set_launch_stage(google_cloud_api::model::LaunchStage::Unspecified),
            );
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = mock_service.expect_create_time_series().await;
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .as_ref()
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = google_cloud_monitoring_v3::model::CreateTimeSeriesRequest::new()
            .set_name("projects/fake_project_id".to_string())
            .set_time_series(vec![google_cloud_monitoring_v3::model::TimeSeries::new()
                .set_metric(
                    google_cloud_api::model::Metric::new()
                        .set_type("workload.googleapis.com/mycounter")
                        .set_labels(HashMap::from([
                            ("float".to_string(), "123.4".to_string()),
                            ("string".to_string(), "string".to_string()),
                            ("int".to_string(), "123".to_string()),
                        ])),
                )
                .set_resource(
                    google_cloud_api::model::MonitoredResource::new()
                        .set_type("k8s_container")
                        .set_labels(HashMap::from([
                            ("location".to_string(), "myavailzone".to_string()),
                            ("cluster_name".to_string(), "mycluster".to_string()),
                            ("container_name".to_string(), "mycontainer".to_string()),
                            ("namespace_name".to_string(), "myns".to_string()),
                            ("pod_name".to_string(), "mypod".to_string()),
                        ])),
                )
                .set_metric_kind(google_cloud_api::model::metric_descriptor::MetricKind::Cumulative)
                .set_value_type(google_cloud_api::model::metric_descriptor::ValueType::Int64)
                .set_points(vec![google_cloud_monitoring_v3::model::Point::new()
                    // .set_interval(
                    //     google_cloud_monitoring_v3::model::TimeInterval::new()
                    //         .set_end_time(google_cloud_wkt::Timestamp::new(1723249032, 972447000).unwrap()),
                    // )
                    .set_value(
                        google_cloud_monitoring_v3::model::TypedValue::new().set_int64_value(12),
                    )])
                .set_unit("myunit")]);
        assert_eq_all_sorted!(create_time_series, expected_create_time_series);
    }
}
