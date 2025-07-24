#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
static THE_RESOURCE: Lazy<Mutex<()>> = Lazy::new(Mutex::default);
#[cfg(test)]
mod tests {
    use crate::gcloud_sdk;
    use crate::gcloud_sdk::google::api::MetricDescriptor;
    use crate::gcloud_sdk::google::monitoring::v3::*;
    use crate::tests::test_cloud_monitoring::THE_RESOURCE;
    use crate::tests::test_utils::*;

    use opentelemetry::metrics::MeterProvider;
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::metrics::{InstrumentKind, StreamBuilder};
    use opentelemetry_sdk::runtime;
    use opentelemetry_sdk::{
        metrics::{periodic_reader_with_async_runtime::PeriodicReader, Aggregation, Instrument, SdkMeterProvider},
        Resource,
    };
    use pretty_assertions_sorted_fork::{assert_eq, assert_eq_all_sorted, assert_eq_sorted};
    use prost::Message;
    use std::collections::HashMap;

    fn my_unit() -> String {
        "myunit".to_string()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_histogram_default_buckets() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myhistogram".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Distribution.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myhistogram".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![
                TimeSeries {
                    metric: Some(
                        gcloud_sdk::google::api::Metric {
                            r#type: "workload.googleapis.com/myhistogram".to_string(),
                            labels: HashMap::from([
                                ("float".to_string(), "123.4".to_string()),
                                ("string".to_string(), "string".to_string()),
                                ("int".to_string(), "123".to_string()),
                            ]),
                        },
                    ),
                    resource: Some(
                        gcloud_sdk::google::api::MonitoredResource {
                            r#type: "generic_node".to_string(),
                            labels: HashMap::from([
                                ("location".to_string(), "global".to_string()),
                                ("namespace".to_string(), "".to_string()),
                                ("node_id".to_string(), "".to_string()),
                            ]),
                        },
                    ),
                    metadata: None,
                    metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                    value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Distribution.into(),
                    points: vec![
                        Point {
                            interval: None,
                            //TODO need to ignore interval for now
                            // interval: Some(
                            //     TimeInterval {
                            //         end_time: Some(
                            //             gcloud_sdk::prost_types::Timestamp {
                            //                 seconds: 1723249032,
                            //                 nanos: 972447000,
                            //             },
                            //         ),
                            //         start_time: Some(
                            //             gcloud_sdk::prost_types::Timestamp {
                            //                 seconds: 1723249032,
                            //                 nanos: 929246000,
                            //             },
                            //         ),
                            //     },
                            // ),
                            value: Some(
                                TypedValue {
                                    value: Some(
                                        gcloud_sdk::google::monitoring::v3::typed_value::Value::DistributionValue(
                                            gcloud_sdk::google::api::Distribution {
                                                count: 10000,
                                                mean: 4999.5,
                                                sum_of_squared_deviation: 0.0,
                                                range: None,
                                                bucket_options: Some(
                                                    gcloud_sdk::google::api::distribution::BucketOptions {
                                                        options: Some(
                                                            gcloud_sdk::google::api::distribution::bucket_options::Options::ExplicitBuckets(
                                                                gcloud_sdk::google::api::distribution::bucket_options::Explicit {
                                                                    bounds: [
                                                                        0.0,
                                                                        5.0,
                                                                        10.0,
                                                                        25.0,
                                                                        50.0,
                                                                        75.0,
                                                                        100.0,
                                                                        250.0,
                                                                        500.0,
                                                                        750.0,
                                                                        1000.0,
                                                                        2500.0,
                                                                        5000.0,
                                                                        7500.0,
                                                                        10000.0,
                                                                    ].to_vec(),
                                                                },
                                                            ),
                                                        ),
                                                    },
                                                ),
                                                bucket_counts: [
                                                    1,
                                                    5,
                                                    5,
                                                    15,
                                                    25,
                                                    25,
                                                    25,
                                                    150,
                                                    250,
                                                    250,
                                                    250,
                                                    1500,
                                                    2500,
                                                    2500,
                                                    2499,
                                                    0,
                                                ].to_vec(),
                                                exemplars: [].to_vec(),
                                            },
                                        ),
                                    ),
                                },
                            ),
                        },
                    ],
                    unit: "myunit".to_string(),
                },
            ],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_histogram_single_bucket() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let exporter = crate::GCPMetricsExporter::fake_new();
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/my_single_bucket_histogram".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Distribution.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "my_single_bucket_histogram".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );

        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![
                TimeSeries {
                    metric: Some(
                        gcloud_sdk::google::api::Metric {
                            r#type: "workload.googleapis.com/my_single_bucket_histogram".to_string(),
                            labels: HashMap::from([
                                ("float".to_string(), "123.4".to_string()),
                                ("string".to_string(), "string".to_string()),
                                ("int".to_string(), "123".to_string()),
                            ]),
                        },
                    ),
                    resource: Some(
                        gcloud_sdk::google::api::MonitoredResource {
                            r#type: "generic_node".to_string(),
                            labels: HashMap::from([
                                ("location".to_string(), "global".to_string()),
                                ("namespace".to_string(), "".to_string()),
                                ("node_id".to_string(), "".to_string()),
                            ]),
                        },
                    ),
                    metadata: None,
                    metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                    value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Distribution.into(),
                    points: vec![
                        Point {
                            interval: None,
                            //TODO need to ignore interval for now
                            // interval: Some(
                            //     TimeInterval {
                            //         end_time: Some(
                            //             gcloud_sdk::prost_types::Timestamp {
                            //                 seconds: 1723249032,
                            //                 nanos: 972447000,
                            //             },
                            //         ),
                            //         start_time: Some(
                            //             gcloud_sdk::prost_types::Timestamp {
                            //                 seconds: 1723249032,
                            //                 nanos: 929246000,
                            //             },
                            //         ),
                            //     },
                            // ),
                            value: Some(
                                TypedValue {
                                    value: Some(
                                        gcloud_sdk::google::monitoring::v3::typed_value::Value::DistributionValue(
                                            gcloud_sdk::google::api::Distribution {
                                                count: 10000,
                                                mean: 4999.5,
                                                sum_of_squared_deviation: 0.0,
                                                range: None,
                                                bucket_options: Some(
                                                    gcloud_sdk::google::api::distribution::BucketOptions {
                                                        options: Some(
                                                            gcloud_sdk::google::api::distribution::bucket_options::Options::ExplicitBuckets(
                                                                gcloud_sdk::google::api::distribution::bucket_options::Explicit {
                                                                    bounds: [
                                                                        5.5,
                                                                    ].to_vec(),
                                                                },
                                                            ),
                                                        ),
                                                    },
                                                ),
                                                bucket_counts: [
                                                    6,
                                                    9994,
                                                ].to_vec(),
                                                exemplars: [].to_vec(),
                                            },
                                        ),
                                    ),
                                },
                            ),
                        },
                    ],
                    unit: "myunit".to_string(),
                },
            ],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_up_down_counter_float() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myupdowncounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myupdowncounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/myupdowncounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::DoubleValue(
                            45.6,
                        )),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_up_down_counter_int() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myupdowncounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myupdowncounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/myupdowncounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(45)),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_up_down_counter_int() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myobservablecounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myobservablecounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/myobservablecounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(45)),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_up_down_counter_float() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myobservablecounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myobservablecounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/myobservablecounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::DoubleValue(
                            45.0,
                        )),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_counter_int() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myobservablecounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myobservablecounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/myobservablecounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(45)),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_counter_float() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myobservablecounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myobservablecounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/myobservablecounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::DoubleValue(
                            45.0,
                        )),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_gauge_int() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myobservablegauge".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myobservablegauge".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/myobservablegauge".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(45)),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_observable_gauge_float() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/myobservablegauge".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "myobservablegauge".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_none(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/myobservablegauge".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Gauge.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::DoubleValue(
                            45.0,
                        )),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_counter_int() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/mycounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "mycounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/mycounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(45)),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_counter_float() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/mycounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "mycounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/mycounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Double.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::DoubleValue(
                            45.0,
                        )),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_invalid_label_keys() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let mycounter = meter
            .u64_counter("mycounter")
            .with_description("foo")
            .with_unit(my_unit());

        let mycounter = mycounter.build();

        mycounter.add(12, &[KeyValue::new("1some.invalid$\\key", "value")]);
        metrics_provider.force_flush().unwrap();
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/mycounter".to_string(),
                labels: vec![gcloud_sdk::google::api::LabelDescriptor {
                    key: "key_1some_invalid__key".to_string(),
                    value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                    description: "".to_string(),
                }],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "mycounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/mycounter".to_string(),
                    labels: HashMap::from([("key_1some_invalid__key".to_string(), "value".to_string())]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "generic_node".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "global".to_string()),
                        ("namespace".to_string(), "".to_string()),
                        ("node_id".to_string(), "".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(12)),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_with_resource() {
        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![
            KeyValue::new("cloud.platform", "gcp_kubernetes_engine"),
            KeyValue::new("cloud.availability_zone", "myavailzone"),
            KeyValue::new("k8s.cluster.name", "mycluster"),
            KeyValue::new("k8s.namespace.name", "myns"),
            KeyValue::new("k8s.pod.name", "mypod"),
            KeyValue::new("k8s.container.name", "mycontainer"),
        ]);
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
        let res = calls.read().await;
        let create_metric_descriptor = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();

        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(MetricDescriptor {
                name: "".to_string(),
                r#type: "workload.googleapis.com/mycounter".to_string(),
                labels: vec![
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "string".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "int".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                    gcloud_sdk::google::api::LabelDescriptor {
                        key: "float".to_string(),
                        value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                        description: "".to_string(),
                    },
                ],
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                unit: "myunit".to_string(),
                description: "foo".to_string(),
                display_name: "mycounter".to_string(),
                metadata: None,
                launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                monitored_resource_types: Vec::new(),
            }),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
                msg
            })
            .collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .start_time
                .is_some(),
            true
        );
        assert_eq!(
            create_time_series.time_series[0].points[0]
                .interval
                .unwrap()
                .end_time
                .is_some(),
            true
        );
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![TimeSeries {
                metric: Some(gcloud_sdk::google::api::Metric {
                    r#type: "workload.googleapis.com/mycounter".to_string(),
                    labels: HashMap::from([
                        ("float".to_string(), "123.4".to_string()),
                        ("string".to_string(), "string".to_string()),
                        ("int".to_string(), "123".to_string()),
                    ]),
                }),
                resource: Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: "k8s_container".to_string(),
                    labels: HashMap::from([
                        ("location".to_string(), "myavailzone".to_string()),
                        ("cluster_name".to_string(), "mycluster".to_string()),
                        ("container_name".to_string(), "mycontainer".to_string()),
                        ("namespace_name".to_string(), "myns".to_string()),
                        ("pod_name".to_string(), "mypod".to_string()),
                    ]),
                }),
                metadata: None,
                metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Int64.into(),
                points: vec![Point {
                    interval: None,
                    //TODO need to ignore interval for now
                    // interval: Some(
                    //     TimeInterval {
                    //         end_time: Some(
                    //             gcloud_sdk::prost_types::Timestamp {
                    //                 seconds: 1723249032,
                    //                 nanos: 972447000,
                    //             },
                    //         ),
                    //         start_time: None,
                    //     },
                    // ),
                    value: Some(TypedValue {
                        value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(12)),
                    }),
                }],
                unit: "myunit".to_string(),
            }],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore]
    async fn test_user_agent() {
        // def test_with_resource(
        //     gcmfake_meter_provider: GcmFakeMeterProvider,
        //     gcmfake: GcmFake,
        // ) -> None:
        //     meter_provider = gcmfake_meter_provider()
        //     counter = meter_provider.get_meter(__name__).create_counter(
        //         "mycounter", description="foo", unit="{myunit}"
        //     )
        //     counter.add(12)
        //     meter_provider.force_flush()

        //     for calls in gcmfake.get_calls().values():
        //         for call in calls:
        //             assert (
        //                 re.match(
        //                     r"^opentelemetry-python \S+; google-cloud-metric-exporter \S+ grpc-python/\S+",
        //                     call.user_agent,
        //                 )
        //                 is not None
        //             )

        let _m = THE_RESOURCE.lock().unwrap();
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(vec![KeyValue::new("service.name", "metric-demo")]);
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
        let res = calls.read().await;
        let create_metric_descriptor_user_agent = res
            .get("CreateMetricDescriptor")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = v.user_agent.clone();
                msg
            })
            .collect::<Vec<String>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor_user_agent = create_metric_descriptor_user_agent.get(0).unwrap().clone();
        println!(
            "create_metric_descriptor_user_agent --> '{}'",
            create_metric_descriptor_user_agent
        );

        assert!(false);

        let create_time_series_user_agent = res
            .get("CreateTimeSeries")
            .unwrap()
            .iter()
            .map(|v| {
                let msg = v.user_agent.clone();
                msg
            })
            .collect::<Vec<String>>();
        let create_time_series_user_agent = create_time_series_user_agent.get(0).unwrap().clone();
        println!("create_time_series_user_agent --> '{}'", create_time_series_user_agent);

        assert!(false);
    }
}
