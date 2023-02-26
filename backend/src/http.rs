use std::net::SocketAddr;

use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::Extension,
    http::{header::CONTENT_TYPE, Method},
    response::{self, IntoResponse},
    routing::get,
    Router,
};

use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::{ApiSchema, Mutation, Query, Subscription},
    task::Task,
};

async fn graphql_handler(schema: Extension<ApiSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("http://localhost:8080/api")
            .subscription_endpoint("ws://localhost:8080/api/ws")
            .finish(),
    )
}

pub async fn listen(task: Task, address: SocketAddr) -> anyhow::Result<()> {
    let schema = Schema::build(Query, Mutation, Subscription)
        .data(task)
        .finish();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .route("/api", get(graphiql).post(graphql_handler))
        .route_service("/api/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema))
        .layer(cors);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    tracing::debug!("listening on {}", address);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
