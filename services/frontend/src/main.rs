use std::fs::File;
use std::io;

use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::extensions::Tracing;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptySubscription, Object, Response, Schema, ServerError, ID};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use frontend::actix_middleware::request_id::RequestIdHeader;
use frontend::graphql::extension::Authorizer;
use frontend::integration::identity_service::schema::{
    AuthenticationOutput, GenerateAccessTokenOutput, GraphQLError, UserAccount,
};
use frontend::integration::identity_service::IdentityServiceRef;
use frontend::schema::authorization::Authorization;
use identity_service::pb::identity_service_client::IdentityServiceClient;
use identity_service::pb::{AuthenticateInput, DescribeAccountInput, GenerateAccessTokenInput, ListAccountsInput};
use service_core::simple_err_map;
use service_core::telemetry::logging::{init_subscriber, make_subscriber};
use thiserror::Error;
use tracing_futures::Instrument;

#[derive(Debug, Error)]
pub enum InitServiceError {
    #[error("Cannot acquire client.")]
    CannotAcquireClient,

    #[error(transparent)]
    IO(#[from] io::Error),
}


#[tokio::main]
#[allow(deprecated)]
async fn main() -> std::result::Result<(), InitServiceError> {
    let subscriber = make_subscriber("frontend", "info");
    init_subscriber(subscriber);

    let schema = create_schema_with_context().await?;

    HttpServer::new(move || {
        App::new()
            .wrap(RequestIdHeader)
            .wrap(tracing_actix_web::TracingLogger::default())
            .configure(configure_service)
            .data(schema.clone())
    })
    .bind("0.0.0.0:8001")?
    .run()
    .await
    .map_err(|e| e.into())
}


pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::post().to(index))
            .route(web::get().guard(guard::Header("upgrade", "websocket")).to(index_ws))
            .route(web::get().to(index_playground)),
    );
}

#[tracing::instrument(skip_all)]
async fn index(schema: web::Data<AppSchema>, http_req: HttpRequest, req: GraphQLRequest) -> GraphQLResponse {
    let authorization = match Authorization::try_from_req(&http_req) {
        Err(e) => {
            tracing::debug!("Cannot extract authorization data: {}", e);

            let permission_denied_error = ServerError::new("Permission denied.", None);
            let response = Response::from_errors(vec![permission_denied_error]);
            return response.into();
        }
        Ok(v) => v,
    };
    let query = req.into_inner().data(authorization);
    schema.execute(query).await.into()
}

async fn index_ws(schema: web::Data<AppSchema>, req: HttpRequest, payload: web::Payload) -> Result<HttpResponse> {
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

#[tracing::instrument]
pub async fn create_schema_with_context() -> std::result::Result<AppSchema, InitServiceError> {
    let identity_service_client = IdentityServiceClient::connect("http://127.0.0.1:8080")
        .await
        .map_err(|_| InitServiceError::CannotAcquireClient)?;

    tracing::info!("Created IdentityService client.");

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .extension(Authorizer)
        .extension(Tracing)
        .data(identity_service_client)
        .finish();

    use std::io::Write;
    let path = "schema";
    let mut output = File::create(path).expect("failed creating schema file");
    write!(output, "{}", &schema.sdl()).expect("failed writing schema");

    Ok(schema)
}

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;
pub struct Query;
pub struct Mutation;

#[Object]
impl Query {
    #[tracing::instrument(skip_all)]
    async fn accounts<'a>(&self, ctx: &Context<'a>) -> std::result::Result<Vec<UserAccount>, ListAccountsError> {
        let mut identity_service_client = ctx.data_unchecked::<IdentityServiceRef>().clone();
        let request = tonic::Request::new(ListAccountsInput {
            include_non_discoverable: true,
            starting_token: None,
            page_size: 32,
        });
        let output = identity_service_client
            .list_accounts(request)
            .instrument(tracing::info_span!("identity_service::list_accounts"))
            .await
            .map_err(simple_err_map!("ListAccounts error.", ListAccountsError::Operation))?
            .into_inner();

        Ok(output
            .accounts
            .into_iter()
            .map(|account| UserAccount {
                id: account.account_id.into(),
                email: account.email,
                first_name: account.first_name,
                last_name: account.last_name,
            })
            .collect())
    }

    #[tracing::instrument(skip_all)]
    async fn account(&self, ctx: &Context<'_>, id: ID) -> std::result::Result<UserAccount, GraphQLError> {
        let mut identity_service_client = ctx.data_unchecked::<IdentityServiceRef>().clone();
        let request = tonic::Request::new(DescribeAccountInput { account_id: id.into() });
        let output = identity_service_client
            .describe_account(request)
            .instrument(tracing::info_span!("identity_service::describe_account"))
            .await
            .map_err(|e| {
                tracing::error!(error = ?&e, "DescribeAccount failed.");
                GraphQLError::Operation(e.into())
            })?
            .into_inner();

        Ok(output
            .account
            .map(|account| UserAccount {
                id: account.account_id.into(),
                email: account.email,
                first_name: account.first_name,
                last_name: account.last_name,
            })
            .expect("malformed response"))
    }
}

#[Object]
impl Mutation {
    #[tracing::instrument(skip_all)]
    async fn authenticate(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> std::result::Result<AuthenticationOutput, AuthenticateError> {
        let mut identity_service_client = ctx.data_unchecked::<IdentityServiceRef>().clone();
        let request = tonic::Request::new(AuthenticateInput { email, password });
        let output = identity_service_client
            .authenticate(request)
            .instrument(tracing::info_span!("identity_service::authenticate"))
            .await
            .map_err(simple_err_map!("Authenticate failed.", AuthenticateError::Operation))?
            .into_inner();

        Ok(AuthenticationOutput {
            access_token: output.access_token,
            refresh_token: output.refresh_token,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn generate_access_token(
        &self,
        ctx: &Context<'_>,
        refresh_token: String,
    ) -> std::result::Result<GenerateAccessTokenOutput, GraphQLError> {
        let mut identity_service_client = ctx.data_unchecked::<IdentityServiceRef>().clone();
        let authorization = ctx
            .data_unchecked::<Option<Authorization>>()
            .as_ref()
            .ok_or(GraphQLError::PermissionDenied)?;
        let request = tonic::Request::new(GenerateAccessTokenInput {
            account_id: authorization.claims.sub.clone(),
            refresh_token,
        });
        let output = identity_service_client
            .generate_access_token(request)
            .await
            .map_err(|_| GraphQLError::PermissionDenied)?
            .into_inner();

        Ok(GenerateAccessTokenOutput {
            access_token: output.access_token,
            refresh_token: output.refresh_token,
        })
    }
}

#[derive(Error, Debug)]
enum ListAccountsError {
    #[error("Operation error.")]
    Operation,
}

#[derive(Error, Debug)]
enum AuthenticateError {
    #[error("Operation error.")]
    Operation,
}
