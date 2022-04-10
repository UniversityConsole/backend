use aws_sdk_dynamodb::Client as RawClient;

#[derive(Debug)]
pub struct Adapter {
    pub(crate) raw: RawClient,
}

impl From<RawClient> for Adapter {
    fn from(raw: RawClient) -> Self {
        Adapter { raw }
    }
}
