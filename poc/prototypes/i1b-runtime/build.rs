fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc_bin_vendored::protoc_bin_path()?);
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    for field in [
        ".rotation.native.v1.RegisterRequest.recovery",
        ".rotation.native.v1.Ack.recovery_receipt",
    ] {
        config.field_attribute(
            field,
            "#[serde(default, skip_serializing_if = \"Option::is_none\")]",
        );
    }
    tonic_prost_build::configure().compile_with_config(
        config,
        &["../../proto/native.proto"],
        &["../../proto"],
    )?;
    println!("cargo:rerun-if-changed=../../proto/native.proto");
    Ok(())
}
