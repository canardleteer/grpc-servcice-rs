pub mod grpc {
    pub mod v1alpha1 {
        tonic::include_proto!("github.canardleteer.grpc_service_rs.v1alpha1");
        pub const TIME_SVC_FILE_DESCRIPTOR_SET: &[u8] =
            tonic::include_file_descriptor_set!("_descriptor");
    }
}
