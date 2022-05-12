pub mod schema;

use identity_service::pb::identity_service_client::IdentityServiceClient;
use tonic::transport::Channel;


pub type IdentityServiceRef = IdentityServiceClient<Channel>;
