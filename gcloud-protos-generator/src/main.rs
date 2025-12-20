#![allow(dead_code, unused_imports, unused_variables, unexpected_cfgs)]

use std::{fs, path::PathBuf};

mod generation;

fn cleanup_genproto_dir(dir: &PathBuf, files_to_keep: &[&str]) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if !files_to_keep.contains(&file_name) {
                            let _ = fs::remove_file(entry.path());
                            // println!("Removed: {}", file_name);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let proto_root = PathBuf::from("gcloud-protos-generator/proto/googleapis");
    let protos = generation::find_proto(proto_root.clone());

    let out_dir = PathBuf::from("src/gcloud_sdk/genproto");
    let _ = fs::remove_dir_all(out_dir.as_path());
    let _ = fs::create_dir(out_dir.as_path());
    // let includes = [proto_root, proto_includes];
    let includes = [proto_root];

    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");
    config.skip_debug(&["."]);

    config.type_attribute(".", "#[derive(Debug)]");

    tonic_prost_build::configure()
        .build_server(true)
        .out_dir(out_dir.clone())
        .compile_with_config(config, &generation::proto_path(&protos), &includes)
        .unwrap();

    cleanup_genproto_dir(
        &out_dir,
        &[
            "google.api.rs",
            "google.monitoring.v3.rs",
            "google.r#type.rs",
            "google.rpc.rs",
        ],
    );
    // let mut out_path = PathBuf::from("gcloud-sdk/src/google_apis.rs");
    // let root = generation::from_protos(protos);
    // fs::write(out_path.clone(), root.gen_code()).unwrap();

    // let input_contents = fs::read_to_string(&out_path).unwrap();
    // let syntax_tree = syn::parse_file(&input_contents).unwrap();
    // let formatted = prettyplease::unparse(&syntax_tree);
    // fs::write(out_path.clone(), formatted).unwrap();

    // out_path.pop();
}
