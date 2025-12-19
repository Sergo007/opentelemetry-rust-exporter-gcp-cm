use super::{utils::kv_map_normalize_k_v, UNIQUE_IDENTIFIER_KEY};
use crate::exporter::to_f64::ToF64;
use crate::gcloud_sdk;
use gcloud_sdk::google::{api::MetricDescriptor, monitoring::v3::TimeSeries};
use opentelemetry_sdk::metrics::data;
use std::time::SystemTime;

pub fn convert<T: ToF64 + Copy>(
    data_point: &data::HistogramDataPoint<T>,
    start_time: &SystemTime,
    time: &SystemTime,
    descriptor: &MetricDescriptor,
    monitored_resource_data: &Option<gcloud_sdk::google::api::MonitoredResource>,
    add_unique_identifier: bool,
    unique_identifier: &str,
) -> TimeSeries {
    let data_point_start_time = start_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let data_point_time = time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let point = gcloud_sdk::google::monitoring::v3::Point {
        interval: Some(gcloud_sdk::google::monitoring::v3::TimeInterval {
            start_time: Some(gcloud_sdk::prost_types::Timestamp {
                seconds: (data_point_start_time / 1_000_000_000) as i64,
                nanos: (data_point_start_time % 1_000_000_000) as i32,
            }),
            end_time: Some(gcloud_sdk::prost_types::Timestamp {
                seconds: (data_point_time / 1_000_000_000) as i64,
                nanos: (data_point_time % 1_000_000_000) as i32,
            }),
        }),
        value: Some(gcloud_sdk::google::monitoring::v3::TypedValue {
            value: Some(
                gcloud_sdk::google::monitoring::v3::typed_value::Value::DistributionValue(
                    gcloud_sdk::google::api::Distribution {
                        count: data_point.count() as i64,
                        mean: {
                            if data_point.count() == 0 {
                                0.0
                            } else {
                                data_point.sum().to_f64() / data_point.count() as f64
                            }
                        },
                        sum_of_squared_deviation: 0.0,
                        bucket_options: Some(gcloud_sdk::google::api::distribution::BucketOptions {
                            options: Some(
                                gcloud_sdk::google::api::distribution::bucket_options::Options::ExplicitBuckets(
                                    gcloud_sdk::google::api::distribution::bucket_options::Explicit {
                                        bounds: data_point.bounds().collect::<Vec<f64>>(),
                                    },
                                ),
                            ),
                        }),
                        range: None,
                        bucket_counts: data_point.bucket_counts().map(|v| v as i64).collect(),
                        exemplars: Default::default(),
                    },
                ),
            ),
        }),
    };

    let mut labels = data_point
        .attributes()
        .map(kv_map_normalize_k_v)
        .collect::<std::collections::HashMap<String, String>>();
    if add_unique_identifier {
        labels.insert(UNIQUE_IDENTIFIER_KEY.to_string(), unique_identifier.to_string());
    }

    let time_series = TimeSeries {
        resource: monitored_resource_data.clone(),
        metadata: None,
        metric_kind: descriptor.metric_kind,
        value_type: descriptor.value_type,
        metric: Some(gcloud_sdk::google::api::Metric {
            r#type: descriptor.r#type.clone(),
            labels: labels,
        }),
        points: vec![point],
        unit: descriptor.unit.clone(),
    };
    time_series
}

#[cfg(feature = "")]
pub fn convert_exponential<T: ToF64 + Copy>(
    data_point: &data::ExponentialHistogramDataPoint<T>,
    descriptor: &MetricDescriptor,
    monitored_resource_data: &Option<gcloud_sdk::google::api::MonitoredResource>,
) -> TimeSeries {
    let data_point_start_time = data_point
        .start_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let data_point_time = data_point
        .time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let point = gcloud_sdk::google::monitoring::v3::Point {
        interval: Some(gcloud_sdk::google::monitoring::v3::TimeInterval {
            start_time: Some(gcloud_sdk::prost_types::Timestamp {
                seconds: (data_point_start_time / 1_000_000_000) as i64,
                nanos: (data_point_start_time % 1_000_000_000) as i32,
            }),
            end_time: Some(gcloud_sdk::prost_types::Timestamp {
                seconds: (data_point_time / 1_000_000_000) as i64,
                nanos: (data_point_time % 1_000_000_000) as i32,
            }),
        }),
        value: Some(gcloud_sdk::google::monitoring::v3::TypedValue {
            value: Some(
                gcloud_sdk::google::monitoring::v3::typed_value::Value::DistributionValue(
                    gcloud_sdk::google::api::Distribution {
                        count: data_point.count as i64,
                        mean: {
                            if data_point.count == 0 {
                                0.0
                            } else {
                                data_point.sum.to_f64() / data_point.count as f64
                            }
                        },
                        sum_of_squared_deviation: 0.0,
                        bucket_options: Some(gcloud_sdk::google::api::distribution::BucketOptions {
                            options: Some(
                                gcloud_sdk::google::api::distribution::bucket_options::Options::ExponentialBuckets(
                                    gcloud_sdk::google::api::distribution::bucket_options::Exponential {
                                        bounds: data_point.bounds.clone(),
                                    },
                                ),
                            ),
                        }),
                        range: None,
                        bucket_counts: data_point.bucket_counts.iter().map(|v| *v as i64).collect(),
                        exemplars: Default::default(),
                    },
                ),
            ),
        }),
    };

    let labels = data_point
        .attributes
        .iter()
        .map(|kv| (normalize_label_key(&kv.key.to_string()), kv.value.to_string()))
        .collect::<std::collections::HashMap<String, String>>();
    // if self.add_unique_identifier {
    //     labels.insert(UNIQUE_IDENTIFIER_KEY.to_string(), self.unique_identifier.clone());
    // }

    let time_series = TimeSeries {
        resource: monitored_resource_data.clone(),
        metadata: None,
        metric_kind: descriptor.metric_kind,
        value_type: descriptor.value_type,
        metric: Some(gcloud_sdk::google::api::Metric {
            r#type: descriptor.r#type.clone(),
            labels: labels,
        }),
        points: vec![point],
        unit: descriptor.unit.clone(),
    };
    time_series
}
