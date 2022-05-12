use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = "proto/service.proto";
    let proto_path: &Path = proto.as_ref();

    // directory the main .proto file resides in
    let proto_dir = proto_path.parent().expect("proto file should reside in a directory");

    tonic_build::configure()
        .out_dir("./src/pb/")
        .compile(&[proto], &[proto_dir])?;

    Ok(())
}
