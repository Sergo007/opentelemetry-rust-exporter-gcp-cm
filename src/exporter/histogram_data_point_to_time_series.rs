use super::{UNIQUE_IDENTIFIER_KEY, utils::kv_map_normalize_k_v};
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

pub fn convert_exponential<T: ToF64 + Copy>(
    data_point: &data::ExponentialHistogramDataPoint<T>,
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

    // Adapted from https://github.com/GoogleCloudPlatform/opentelemetry-operations-go/blob/v1.8.0/exporter/collector/metrics.go#L582

    // Calculate underflow bucket (zero count + negative buckets)
    let mut underflow = data_point.zero_count();
    underflow += data_point.negative_bucket().counts().sum::<u64>();

    // Create bucket counts array: [underflow, positive_buckets..., overflow=0]
    let mut bucket_counts = vec![underflow as i64];
    bucket_counts.extend(data_point.positive_bucket().counts().map(|v| v as i64));
    bucket_counts.push(0); // overflow bucket is always empty

    // Determine bucket options
    let bucket_options = if data_point.positive_bucket().counts().next().is_none() {
        // If no positive buckets, use explicit buckets with bounds=[0]
        google_cloud_api::model::distribution::BucketOptions::new().set_explicit_buckets(
            google_cloud_api::model::distribution::bucket_options::Explicit::new().set_bounds(vec![0.0]),
        )
    } else {
        // Use exponential bucket options
        // growth_factor = 2^(2^(-scale))
        let growth_factor = f64::powf(2.0, f64::powf(2.0, -(data_point.scale() as f64)));
        // scale = growth_factor^(positive_bucket_offset)
        let scale = f64::powf(growth_factor, data_point.positive_bucket().offset() as f64);
        let num_finite_buckets = (bucket_counts.len() - 2) as i32;

        google_cloud_api::model::distribution::BucketOptions::new().set_exponential_buckets(
            google_cloud_api::model::distribution::bucket_options::Exponential::new()
                .set_num_finite_buckets(num_finite_buckets)
                .set_growth_factor(growth_factor)
                .set_scale(scale),
        )
    };

    let mean = if data_point.count() == 0 {
        0.0
    } else {
        data_point.sum().to_f64() / data_point.count() as f64
    };

    let distribution = google_cloud_api::model::Distribution::new()
        .set_count(data_point.count() as i64)
        .set_mean(mean)
        .set_sum_of_squared_deviation(0.0)
        .set_bucket_options(bucket_options)
        .set_bucket_counts(bucket_counts);

    point.value = Some(google_cloud_monitoring_v3::model::TypedValue::new().set_distribution_value(distribution));

    let mut labels = data_point
        .attributes()
        .map(|kv| (kv_map_normalize_k_v(kv).0, kv_map_normalize_k_v(kv).1))
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
        .set_unit(descriptor.unit.clone());

    if let Some(resource) = monitored_resource_data {
        time_series = time_series.set_resource(resource.clone());
    }

    time_series
}
