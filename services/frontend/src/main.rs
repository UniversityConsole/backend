use actix_web::guard;
use actix_web::web;
use actix_web::App;
use actix_web::{HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Enum, Object, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum State {
    Active,
    Inactive,
}

#[derive(Clone)]
pub struct StorageObject {
    pub path: String,
    pub state: State,
}

impl StorageObject {
    fn new(path: impl Into<String>, state: State) -> Self {
        StorageObject {
            path: path.into(),
            state,
        }
    }
}

#[Object]
impl StorageObject {
    async fn path(&self) -> &String {
        &self.path
    }

    async fn state(&self) -> &State {
        &self.state
    }
}

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
    let ws_subscription =
        async_graphql_actix_web::GraphQLSubscription::new(Schema::clone(&*schema));
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
    let objects = vec![
        StorageObject::new("/", State::Active),
        StorageObject::new("/home", State::Active),
        StorageObject::new("/home/vicbarbu", State::Active),
        StorageObject::new("/home/out", State::Inactive),
    ];

    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(objects)
        .finish()
}

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;
pub struct Query;

#[Object]
impl Query {
    async fn objects(&self, ctx: &Context<'_>) -> Vec<StorageObject> {
        ctx.data::<Vec<StorageObject>>()
            .expect("Can't get objects.")
            .to_vec()
    }

    async fn version(&self, ctx: &Context<'_>) -> u32 {
        1
    }
}
