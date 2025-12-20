# GCloud Protos Generator

A build tool for generating Rust bindings from Google Cloud Platform Protocol Buffer definitions.

## Purpose

This generator compiles Protocol Buffer definitions from the Google APIs repository into Rust code using `prost` and `tonic`. It specifically generates the necessary types and gRPC client code for Google Cloud Monitoring API.

## What It Does

1. **Finds Proto Files**: Scans the `gcloud-protos-generator/proto/googleapis` directory for `.proto` files
2. **Generates Rust Code**: Uses `prost-build` and `tonic-build` to compile proto files into Rust code
3. **Outputs to**: `src/gcloud_sdk/genproto/` directory
4. **Cleanup**: Removes all generated files except the required ones:
   - `google.api.rs`
   - `google.monitoring.v3.rs`
   - `google.r#type.rs`
   - `google.rpc.rs`

## Usage

Run the generator from the project root:

```bash
cargo run --bin gcloud-protos-generator
```

Or if you're in the gcloud-protos-generator directory:

```bash
cargo run
```

## Inspired
- [gcloud-sdk-rs](https://github.com/abdolence/gcloud-sdk-rs)