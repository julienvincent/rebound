use anyhow::Context;
use axum::Json;
use axum::extract::State;
use schemars::JsonSchema;

use crate::error::ApiError;
use crate::router::context;
use crate::system::SystemApi;

use super::{Instance, row_to_instance};

#[derive(Debug, serde::Deserialize, JsonSchema, aide::OperationIo)]
pub struct GetRequest {
  pub id: String,
}

pub async fn get(
  State(system): State<context::Context>,
  Json(body): Json<GetRequest>,
) -> Result<Json<Instance>, ApiError> {
  let client = system
    .pool()
    .get()
    .await
    .context("failed to get db connection")?;

  let row = client
    .query_opt("SELECT * FROM rebound.instance WHERE id = $1", &[&body.id])
    .await
    .context("failed to query instance")?;

  let row = row.ok_or_else(|| ApiError::not_found("instance not found"))?;
  Ok(Json(row_to_instance(&row)))
}
