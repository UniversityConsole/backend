mod authorization;
mod integration;

use crate::integration::identity_service::client::identity_service_client::IdentityServiceClient;
use crate::integration::identity_service::client::ListAccountsInput;
use crate::integration::identity_service::schema::UserAccount;
use actix_web::guard;
use actix_web::web;
use actix_web::App;
use actix_web::{HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, ID};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let schema = create_schema_with_context();

    HttpServer::new(move || App::new().configure(configure_service).data(schema.clone()))
        .bind("0.0.0.0:8001")?
        .run()
        .await
}

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::post().to(index))
            .route(
                web::get()
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
            .route(web::get().to(index_playground)),
    );
}

async fn index(
    schema: web::Data<AppSchema>,
    _http_req: HttpRequest,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let query = req.into_inner();
    schema.execute(query).await.into()
}

async fn index_ws(
    schema: web::Data<AppSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    use async_graphql_actix_web::GraphQLSubscription;

    let ws_subscription = GraphQLSubscription::new(Schema::clone(&*schema));
    ws_subscription.start(&req, payload)
}

async fn index_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        ))
}

pub fn create_schema_with_context() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(authorization::Authorizer)
        .finish()
}

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;
pub struct Query;

#[Object]
impl Query {
    async fn accounts(
        &self,
        _ctx: &Context<'_>,
    ) -> std::result::Result<Vec<UserAccount>, ListAccountsError> {
        let mut identity_service_client = IdentityServiceClient::connect("http://127.0.0.1:8080")
            .await
            .map_err(|_| ListAccountsError::CannotAcquireClient)?;
        let request = tonic::Request::new(ListAccountsInput {
            include_non_discoverable: true,
            starting_token: None,
            page_size: 32,
        });
        let output = identity_service_client
            .list_accounts(request)
            .await
            .map_err(|_| ListAccountsError::Operation)?
            .into_inner();

        Ok(output
            .accounts
            .into_iter()
            .map(|account| UserAccount {
                account_id: account.account_id.into(),
                email: account.email,
                first_name: account.first_name,
                last_name: account.last_name,
            })
            .collect())
    }

    async fn api_version(&self, _ctx: &Context<'_>) -> u32 {
        1
    }
}

#[derive(thiserror::Error, Debug)]
enum ListAccountsError {
    #[error("Cannot acquire IdentityService client.")]
    CannotAcquireClient,

    #[error("Operation error.")]
    Operation,
}
