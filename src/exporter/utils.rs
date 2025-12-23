use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};

use std::collections::HashSet;

pub(crate) fn log_warning(err: String) {
    tracing::warn!("{}", err);
}

pub(crate) fn get_data_points_attributes_keys(data: &AggregatedMetrics) -> HashSet<String> {
    let attributes_keys: Vec<String> = match data {
        AggregatedMetrics::F64(v) => match v {
            MetricData::Histogram(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::ExponentialHistogram(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::Sum(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::Gauge(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
        },
        AggregatedMetrics::I64(v) => match v {
            MetricData::Histogram(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::ExponentialHistogram(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::Sum(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::Gauge(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
        },
        AggregatedMetrics::U64(v) => match v {
            MetricData::Histogram(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::ExponentialHistogram(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::Sum(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
            MetricData::Gauge(m) => m
                .data_points()
                .map(|point| point.attributes().map(kv_map_k))
                .flatten()
                .collect(),
        },
    };
    HashSet::from_iter(attributes_keys.into_iter())
}

use unicode_segmentation::UnicodeSegmentation;

///Makes the key into a valid GCM label key
///
///    See reference impl
///
///    https://github.com/GoogleCloudPlatform/opentelemetry-operations-go/blob/e955c204f4f2bfdc92ff0ad52786232b975efcc2/exporter/metric/metric.go#L595-L604
///
pub(crate) fn normalize_label_key(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    let s = sanitize_string(s);
    if s.chars().next().map_or(false, |c| c.is_digit(10)) {
        return format!("key_{}", s);
    }
    s
}

// Converts anything that is not a letter or digit to an underscore
fn sanitize_string(s: &str) -> String {
    s.graphemes(true)
        .map(|g| {
            if g.chars().all(|c| c.is_alphanumeric()) {
                g.to_string()
            } else {
                "_".to_string()
            }
        })
        .collect::<String>()
}

pub(crate) fn kv_map_normalize_k_v(kv: &KeyValue) -> (String, String) {
    (normalize_label_key(&kv.key.to_string()), kv.value.to_string())
}

pub(crate) fn kv_map_k(kv: &KeyValue) -> String {
    kv.key.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_label_key() {
        assert_eq!(normalize_label_key("valid_key_1"), "valid_key_1");
        assert_eq!(normalize_label_key("hellø"), "hellø");
        assert_eq!(normalize_label_key("123"), "key_123");
        assert_eq!(normalize_label_key("key!321"), "key_321");
        assert_eq!(normalize_label_key("hyphens-dots.slashes/"), "hyphens_dots_slashes_");
        assert_eq!(normalize_label_key("non_letters_:£¢$∞"), "non_letters______");
    }
}
