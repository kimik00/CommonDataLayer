fn main() -> std::io::Result<()> {
    tonic_build::configure().compile(
        &[
            "proto/blobstore.proto",
            "proto/docstore.proto",
            "proto/query.proto",
            "proto/registry.proto",
        ],
        &["proto/"],
    )
}
