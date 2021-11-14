mod context;
mod operations;
mod svc;

use context::Context;
use log::LevelFilter;
use operations::create_account::create_account;
use rusoto_core::Region;
use rusoto_dynamodb::DynamoDbClient;
use simple_logger::SimpleLogger;
use svc::identity_service_server::IdentityService;
use svc::identity_service_server::IdentityServiceServer;
use svc::CreateAccountInput;
use svc::CreateAccountOutput;
use tonic::transport::Server;
use tonic::Request;
use tonic::Response;
use tonic::Status;

#[derive(Debug)]
struct IdentityServiceImpl;

#[tonic::async_trait]
impl IdentityService for IdentityServiceImpl {
    async fn create_account(
        &self,
        request: Request<CreateAccountInput>,
    ) -> Result<Response<CreateAccountOutput>, Status> {
        let ctx = Context {
            dynamodb_client: Box::new(DynamoDbClient::new(Region::Custom {
                name: "eu-west-1".to_string(),
                endpoint: Context::key(&context::ContextKey::DynamoDbEndpoint),
            })),
            accounts_table_name: Context::key(&context::ContextKey::AccountsTableName),
        };

        create_account(&ctx, request.get_ref())
            .await
            .map(|output| Response::new(output))
            .map_err(|err| err.into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level(module_path!(), LevelFilter::Debug)
        .init()
        .unwrap();

    let addr = "[::1]:8080".parse().unwrap();
    let identity_service = IdentityServiceImpl {};
    let server = IdentityServiceServer::new(identity_service);

    Server::builder().add_service(server).serve(addr).await?;

    log::info!("Listening on {}", addr);

    Ok(())
}
