use axum::http::Method;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, Router};
use axum::Json;
use env_logger::Env;
use http::header::HeaderName;
use log::info;
use tower_http::cors::CorsLayer;

use slurpgit::repositories;

//TODO: shut down properly
#[tokio::main]
async fn main() {
    // Q: how to get axum server to log startup etc?
    env_logger::Builder::from_env(Env::default()).init();

    let address = &"0.0.0.0:8080".parse().unwrap();
    info!("listening on {:?}", address);

    let hx_headers: [HeaderName; 2] = [
        "HX-Current-URL".parse().unwrap(),
        "HX-Request".parse().unwrap(),
    ];
    let origins = [
        "http://localhost:8000".parse().unwrap(),
        "http://127.0.0.1:8000".parse().unwrap(),
        "http://49.186.114.143".parse().unwrap(),
        "http://crisbennett.com".parse().unwrap(),
        "https://crisbennett.com".parse().unwrap(),
        "http://www.crisbennett.com".parse().unwrap(),
        "https://www.crisbennett.com".parse().unwrap(),
    ];
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(hx_headers)
        .allow_origin(origins);

    let routes = Router::new()
        .route("/test", get(test))
        .route("/repositories/json", get(get_repos))
        .route("/repositories", get(get_repos_html));
    let app = Router::new().merge(routes).layer(cors);

    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn test() -> impl IntoResponse {
    Html("tested")
}

async fn get_repos() -> impl IntoResponse {
    let repos = repositories().await.unwrap();
    Json(repos)
}

// TODO: provide css class names via api
async fn get_repos_html() -> impl IntoResponse {
    let repos = repositories().await.unwrap();

    // TODO: Find a way to go from structs -> axum Bytes. or via tera templates

    let mut response = String::from(r#"<div class="repo-table-container">"#);
    for heading in ["Name", "Description", "Language", "Type", "Github Url"] {
        response += r#"<div class="repo-table-header">"#;
        response += heading;
        response += "</div>";
    }
    for repo in &repos {
        // a private repo with no description is useless as we obfuscante name & url
        if repo.private && repo.description.is_none() {
            continue;
        };
        response += r#"<div class="repo-table-item">"#;
        response += &repo.name;
        response += "</div>";
        response += r#"<div class="repo-table-item">"#;
        response += &repo.description.clone().unwrap_or(String::new());
        response += "</div>";
        response += r#"<div class="repo-table-item">"#;
        response += &repo.language.clone().unwrap_or("?".into());
        response += "</div>";
        response += r#"<div class="repo-table-item">"#;
        response += &repo.repo_type.to_string();
        response += "</div>";
        response += r#"<div class="repo-table-item">"#;
        response += &repo.url_anchor();
        response += "</div>";
    }
    response += "</div>";
    response += "</div>";

    Html(response)
}
