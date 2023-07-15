use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
struct AppState {
    db: Surreal<Client>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: "test",
        password: "test",
    })
    .await?;

    // Select a specific namespace / database
    db.use_ns("axum_test").use_db("axum_test").await?;

    let state = AppState { db };
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 7878));

    axum::Server::bind(&addr)
        .serve(app(state).into_make_service())
        .await
        .context("error running HTTP server")
}
fn app(state: AppState) -> Router {
    Router::new().route("/", get(root)).with_state(state)
}

async fn root(State(state): State<AppState>) -> Result<impl IntoResponse> {
    dbg!(&state.db);
    Ok(Html("<h1>ok</h1>").into_response())
}
