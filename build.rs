use tonic_build;

fn main() {
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["proto/api.proto"], &["proto"])
        .unwrap();
    // tonic_build::configure()
    //     .compile(&["proto/api.proto"], &["proto"])?;
}
