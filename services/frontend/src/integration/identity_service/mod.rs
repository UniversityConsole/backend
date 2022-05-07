pub mod client;
pub mod schema;

use tonic::transport::Channel;

use self::client::identity_service_client::IdentityServiceClient;


pub type IdentityServiceRef = IdentityServiceClient<Channel>;
