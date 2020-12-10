pub mod schema_registry {
    tonic::include_proto!("registry");
}

pub mod document_store {
    tonic::include_proto!("docstore");
}

pub mod blob_store {
    tonic::include_proto!("blobstore");
}

pub mod query_service {
    tonic::include_proto!("query");
}
