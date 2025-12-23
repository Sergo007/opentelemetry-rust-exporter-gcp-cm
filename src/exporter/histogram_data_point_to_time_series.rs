use super::{utils::kv_map_normalize_k_v, UNIQUE_IDENTIFIER_KEY};
use crate::exporter::to_f64::ToF64;
use opentelemetry_sdk::metrics::data;
use std::time::SystemTime;

pub fn convert<T: ToF64 + Copy>(
    data_point: &data::HistogramDataPoint<T>,
    start_time: &SystemTime,
    time: &SystemTime,
    descriptor: &google_cloud_api::model::MetricDescriptor,
    monitored_resource_data: &Option<google_cloud_api::model::MonitoredResource>,
    add_unique_identifier: bool,
    unique_identifier: &str,
) -> google_cloud_monitoring_v3::model::TimeSeries {
    let mut point = google_cloud_monitoring_v3::model::Point::new();
    let mut interval = google_cloud_monitoring_v3::model::TimeInterval::new();

    let data_point_start_time = start_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    interval.start_time = Some(
        google_cloud_wkt::Timestamp::new(
            (data_point_start_time / 1_000_000_000) as i64,
            (data_point_start_time % 1_000_000_000) as i32,
        )
        .unwrap(),
    );

    let data_point_time = time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    interval.end_time = Some(
        google_cloud_wkt::Timestamp::new(
            (data_point_time / 1_000_000_000) as i64,
            (data_point_time % 1_000_000_000) as i32,
        )
        .unwrap(),
    );

    point.interval = Some(interval);

    let distribution = google_cloud_api::model::Distribution::new()
        .set_count(data_point.count() as i64)
        .set_mean(if data_point.count() == 0 {
            0.0
        } else {
            data_point.sum().to_f64() / data_point.count() as f64
        })
        .set_sum_of_squared_deviation(0.0)
        .set_bucket_options(
            google_cloud_api::model::distribution::BucketOptions::new().set_explicit_buckets(
                google_cloud_api::model::distribution::bucket_options::Explicit::new().set_bounds(data_point.bounds()),
            ),
        )
        .set_bucket_counts(data_point.bucket_counts().map(|v| v as i64));

    point.value = Some(google_cloud_monitoring_v3::model::TypedValue::new().set_distribution_value(distribution));

    let mut labels = data_point
        .attributes()
        .map(kv_map_normalize_k_v)
        .collect::<std::collections::HashMap<String, String>>();
    if add_unique_identifier {
        labels.insert(UNIQUE_IDENTIFIER_KEY.to_string(), unique_identifier.to_string());
    }

    let mut time_series = google_cloud_monitoring_v3::model::TimeSeries::new()
        .set_metric_kind(descriptor.metric_kind.clone())
        .set_value_type(descriptor.value_type.clone())
        .set_metric(
            google_cloud_api::model::Metric::new()
                .set_type(descriptor.r#type.clone())
                .set_labels(labels),
        )
        .set_points(vec![point])
        .set_unit(descriptor.unit.clone())
        .set_description("".to_string());

    if let Some(resource) = monitored_resource_data {
        time_series = time_series.set_resource(resource.clone());
    }

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
