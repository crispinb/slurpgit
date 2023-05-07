use axum::response::IntoResponse;
use axum::routing::{get, Router};
use axum::Json;
use log::info;
use env_logger::Env;

use slurpgit::repositories;

#[tokio::main]
async fn main() {
    // Q: how to get axum server to log startup etc?
    env_logger::Builder::from_env(Env::default()).init();

    let address = &"0.0.0.0:8080".parse().unwrap();
    info!("listening on {:?}", address);

    axum::Server::bind(address)
        .serve(
            Router::new()
                .route("/repositories", get(get_repos))
                .into_make_service(),
        )
        .await
        .unwrap();
}

async fn get_repos() -> impl IntoResponse {
    let repos = repositories().await.unwrap();
    Json(repos)
}
