fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only compile protos if the proto directory exists (for examples)
    let proto_dir = std::path::Path::new("proto");
    if proto_dir.exists() {
        let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

        tonic_build::configure()
            .file_descriptor_set_path(out_dir.join("greeter_descriptor.bin"))
            .compile_protos(&["proto/greeter.proto"], &["proto"])?;
    }

    Ok(())
}
