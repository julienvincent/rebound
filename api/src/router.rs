use aide::axum::{
  ApiRouter,
  routing::{get, post},
};
use axum::{
  body::Body,
  http::Request,
  middleware::{Next, from_fn},
  response::Response,
};
use tower_http::cors::CorsLayer;

pub mod context;

use crate::{routes, system::SystemApi};

async fn livez() -> &'static str {
  "ok"
}

pub fn build_router(system: impl SystemApi + Send + Sync + 'static) -> ApiRouter {
  ApiRouter::new()
    .route("/livez", get(livez))
    .api_route("/region/list", post(routes::region::list_regions))
    .api_route("/instance/create", post(routes::instance::create))
    .api_route("/instance/stop", post(routes::instance::stop))
    .api_route("/instance/get", post(routes::instance::get))
    .api_route("/instance/list", post(routes::instance::list))
    .api_route("/usage/summary", post(routes::usage::summary))
    .api_route("/usage/instances", post(routes::usage::instances))
    .layer(from_fn(log_server_errors))
    .layer(CorsLayer::permissive())
    .with_state(context::Context::new(system))
}

async fn log_server_errors(req: Request<Body>, next: Next) -> Response {
  let method = req.method().clone();
  let path = req.uri().path().to_owned();

  let response = next.run(req).await;
  if response.status().is_success() {
    log::info!("{} {} -> {}", method, path, response.status());
  } else {
    log::error!("{} {} -> {}", method, path, response.status());
  }

  response
}
