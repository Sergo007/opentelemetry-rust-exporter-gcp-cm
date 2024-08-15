//! run with `$ cargo run --example basic --all-features
use opentelemetry::{metrics::MeterProvider as _, KeyValue};
use opentelemetry_resourcedetector_gcp_rust::GoogleCloudResourceDetector;
use opentelemetry_rust_exporter_gcp_cm::{
    GCPMetricsExporter, GCPMetricsExporterConfig, MonitoredResourceDataConfig,
};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    runtime, Resource,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
fn to_labels(kv: serde_json::Value) -> HashMap<String, String> {
    kv.as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.to_string(), v.as_str().unwrap().to_string()))
        .collect()
}
async fn init_metrics() -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/Users/serhiiyatsina/projects/cybx/opentelemetry/opentelemetry-rust-exporter-gcp-cm/.secrets/977645940426-compute@developer.gserviceaccount.com.json");

    let mut cfg = GCPMetricsExporterConfig::default();
    cfg.prefix = "custom.googleapis.com/opencensus/cybx.io/test_service".to_string();
    cfg.custom_monitored_resource_data = Some(
        // https://cloud.google.com/monitoring/api/resources#tag_global
        MonitoredResourceDataConfig {
            r#type: "global".to_string(),
            labels: to_labels(json!({
                "project_id": "cybx-chat",
            })),
        },
    );
    let exporter = GCPMetricsExporter::new(cfg).await?;
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    let _gcp_detector = GoogleCloudResourceDetector::new().await;
    // https://cloud.google.com/monitoring/api/resources#tag_global
    let rname = Resource::new(vec![KeyValue::new("service.name", "metric-from-rust")]);
    let res = Resource::default().merge(&rname);
    // println!("{:#?}", res);
    // )]).merge(&gcp_detector.get_resource());
    Ok(SdkMeterProvider::builder()
        .with_resource(res)
        .with_reader(reader)
        .build())
}

#[tokio::main]
#[allow(unused_must_use)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meter_provider = init_metrics().await?;
    println!("start metrics");

    let meter = meter_provider.meter("user-event-test");

    // Create a Counter Instrument.
    let counter = meter
        .f64_counter("counter_f64_test")
        .with_description("test_decription")
        .with_unit("test_unit")
        .init();

    let counter2 = meter
        .u64_counter("counter_u64_test")
        .with_description("test_decription")
        .with_unit("test_unit")
        .init();

    // Create an UpDownCounter Instrument.
    let updown_counter = meter.i64_up_down_counter("up_down_counter_i64_test").init();
    let updown_counter2 = meter.f64_up_down_counter("up_down_counter_f64_test").init();

    // Create a Histogram Instrument.
    let histogram = meter
        .f64_histogram("histogram_f64_test")
        .with_description("test_description")
        .init();
    let histogram2 = meter
        .u64_histogram("histogram_u64_test")
        .with_description("test_description")
        .init();

    // Create a ObservableGauge instrument and register a callback that reports the measurement.
    let gauge = meter
        .f64_observable_gauge("observable_gauge_f64_test")
        .with_unit("test_unit")
        .with_description("test_description")
        .init();

    let gauge2 = meter
        .u64_observable_gauge("observable_gauge_u64_test")
        .with_unit("test_unit")
        .with_description("test_description")
        .init();

    meter.register_callback(&[gauge.as_any()], move |observer| {
        observer.observe_f64(
            &gauge,
            1.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })?;

    meter.register_callback(&[gauge2.as_any()], move |observer| {
        observer.observe_u64(
            &gauge2,
            1,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })?;

    // Create a ObservableCounter instrument and register a callback that reports the measurement.
    let observable_counter = meter
        .u64_observable_counter("observable_counter_u64_test")
        .with_description("test_description")
        .with_unit("test_unit")
        .init();

    let observable_counter2 = meter
        .f64_observable_counter("observable_counter_f64_test")
        .with_description("test_description")
        .with_unit("test_unit")
        .init();

    meter.register_callback(&[observable_counter.as_any()], move |observer| {
        observer.observe_u64(
            &observable_counter,
            100,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })?;

    meter.register_callback(&[observable_counter2.as_any()], move |observer| {
        observer.observe_f64(
            &observable_counter2,
            100.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })?;

    // Create a Observable UpDownCounter instrument and register a callback that reports the measurement.
    let observable_up_down_counter = meter
        .i64_observable_up_down_counter("observable_up_down_counter_i64_test")
        .with_description("test_description")
        .with_unit("test_unit")
        .init();
    let observable_up_down_counter2 = meter
        .f64_observable_up_down_counter("observable_up_down_counter_f64_test")
        .with_description("test_description")
        .with_unit("test_unit")
        .init();

    meter.register_callback(&[observable_up_down_counter.as_any()], move |observer| {
        observer.observe_i64(
            &observable_up_down_counter,
            100,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })?;

    meter.register_callback(&[observable_up_down_counter2.as_any()], move |observer| {
        observer.observe_f64(
            &observable_up_down_counter2,
            100.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })?;

    loop {
        // Record measurements using the Counter instrument.
        counter.add(
            1.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );

        counter2.add(
            1,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );

        // Record measurements using the UpCounter instrument.
        updown_counter.add(
            10,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );

        updown_counter2.add(
            10.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );

        // Record measurements using the histogram instrument.
        histogram.record(
            10.5,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );
        histogram2.record(
            10,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );
        // println!("recorded metrics");
        // Sleep for 0.1 second
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

// fn main() {
//     println!("Hello, world!");
// }