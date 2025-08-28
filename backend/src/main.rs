mod auth;
mod schema;
mod db;

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    response::{IntoResponse, Html},
    http::{StatusCode, HeaderMap},
};
use async_graphql::{Schema, EmptySubscription};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use async_graphql::http::GraphiQLSource;
use tower_http::{
    cors::{CorsLayer, Any},
    services::ServeDir,
};
use tracing_subscriber::EnvFilter;

use schema::{AppSchema, QueryRoot, MutationRoot, AppState};

#[derive(Clone)]
struct AppCtx {
    schema: AppSchema,
    state: AppState,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://backend/app.db".into());

    let pool = db::pool(&database_url).await?;

    let state = schema::AppState {
        pool: pool.clone(),
        jwt_secret,
    };

    let schema: AppSchema =
        Schema::build(QueryRoot, MutationRoot, EmptySubscription)
            .data(state.clone())
            .finish();

    let ctx = AppCtx { schema: schema.clone(), state: state.clone() };

    let static_files = ServeDir::new("static").append_index_html_on_directories(true);

    let app = Router::new()
        // GraphQL API + GraphiQL UI
        .route("/graphql", post(graphql_handler).get(graphiql))
        // Locked REST endpoint (JWT required)
        .route("/secret", get(secret_handler))
        // Serve static site at /
        .fallback_service(static_files)
        // CORS
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(ctx);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("GraphiQL → http://localhost:3000/graphiql");
    println!("Demo page → http://localhost:3000/");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

// Inject the incoming request headers into the GraphQL context
async fn graphql_handler(
    State(ctx): State<AppCtx>,
    headers: HeaderMap,        // <-- non-body extractor(s) first
    req: GraphQLRequest,       // <-- body extractor LAST
) -> GraphQLResponse {
    ctx.schema
        .execute(req.into_inner().data(headers)) // inject headers into GQL context
        .await
        .into()
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn secret_handler(
    State(ctx): State<AppCtx>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(authz) = headers.get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok()) else {
        return (StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string());
    };
    let Some(token) = authz.strip_prefix("Bearer ") else {
        return (StatusCode::UNAUTHORIZED, "Invalid Authorization header".to_string());
    };
    match crate::auth::parse_jwt(&ctx.state.jwt_secret, token) {
        Ok(user_id) => {
            let msg = format!(r#"{{"message":"Welcome, user {}. This is the locked page."}}"#, user_id);
            (StatusCode::OK, msg)
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string()),
    }
}
