tonic::include_proto!("identity_service");

impl From<service_core::resource_access::AccessRequest> for AccessRequest {
    fn from(val: service_core::resource_access::AccessRequest) -> Self {
        use service_core::resource_access::AccessKind as AccessKindModel;

        use self::access_request::AccessKind;

        AccessRequest {
            access_kind: match val.kind {
                AccessKindModel::Query => AccessKind::Query,
                AccessKindModel::Mutation => AccessKind::Mutation,
            } as i32,
            paths: val.paths.into_iter().map(|v| v.to_string()).collect(),
        }
    }
}
