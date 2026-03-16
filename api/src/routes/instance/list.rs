use anyhow::Context;
use axum::Json;
use axum::extract::State;
use schemars::JsonSchema;

use crate::error::ApiError;
use crate::router::context;
use crate::system::SystemApi;

use super::{Instance, row_to_instance};

#[derive(Debug, serde::Deserialize, JsonSchema, aide::OperationIo)]
pub struct ListRequest {
  pub status: Option<String>,
  pub region: Option<String>,
  pub limit: Option<i64>,
  pub cursor: Option<String>,
}

#[derive(Debug, serde::Serialize, JsonSchema, aide::OperationIo)]
pub struct ListResponse {
  pub instances: Vec<Instance>,
  pub next_cursor: Option<String>,
}

pub async fn list(
  State(system): State<context::Context>,
  Json(body): Json<ListRequest>,
) -> Result<Json<ListResponse>, ApiError> {
  let limit = body.limit.unwrap_or(50).min(100);

  let client = system
    .pool()
    .get()
    .await
    .context("failed to get db connection")?;

  let rows = client
    .query(
      "SELECT *
       FROM rebound.instance
       WHERE ($1::text IS NULL OR status = $1)
         AND ($2::text IS NULL OR region = $2)
         AND ($3::text IS NULL OR id < $3)
       ORDER BY id DESC
       LIMIT $4",
      &[&body.status, &body.region, &body.cursor, &limit],
    )
    .await
    .context("failed to list instances")?;

  let instances: Vec<Instance> = rows.iter().map(row_to_instance).collect();

  let next_cursor = if instances.len() == limit as usize {
    instances.last().map(|i| i.id.clone())
  } else {
    None
  };

  Ok(Json(ListResponse {
    instances,
    next_cursor,
  }))
}
