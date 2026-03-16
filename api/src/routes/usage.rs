use anyhow::Context;
use axum::Json;
use axum::extract::State;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;

use crate::error::ApiError;
use crate::router::context;
use crate::system::SystemApi;

// --- POST /usage/summary ---

#[derive(Debug, serde::Deserialize, JsonSchema, aide::OperationIo)]
pub struct SummaryRequest {
  pub from: Option<DateTime<Utc>>,
  pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize, JsonSchema)]
pub struct RegionSummary {
  pub region: String,
  pub instances: i64,
  pub duration_secs: f64,
  pub cost_usd: f64,
}

#[derive(Debug, serde::Serialize, JsonSchema, aide::OperationIo)]
pub struct SummaryResponse {
  pub total_instances: i64,
  pub total_duration_secs: f64,
  pub total_cost_usd: f64,
  pub by_region: Vec<RegionSummary>,
}

pub async fn summary(
  State(system): State<context::Context>,
  Json(body): Json<SummaryRequest>,
) -> Result<Json<SummaryResponse>, ApiError> {
  let client = system
    .pool()
    .get()
    .await
    .context("failed to get db connection")?;

  let sql = "SELECT
               region,
               count(*) AS instances,
               coalesce(sum(
                 extract(epoch FROM (stopped_at - started_at))::double precision
               ), 0.0::double precision) AS duration_secs,
               coalesce(sum(
                 (cost_per_sec_usd * extract(epoch FROM (stopped_at - started_at)))::double precision
               ), 0.0::double precision) AS cost_usd
              FROM
                rebound.instance
              WHERE ($1::timestamptz IS NULL
                OR created_at >= $1)
              AND ($2::timestamptz IS NULL
                OR created_at <= $2)
              AND started_at IS NOT NULL
              AND stopped_at IS NOT NULL
              GROUP BY
                region
             ORDER BY
               cost_usd DESC";

  let rows = client
    .query(sql, &[&body.from, &body.to])
    .await
    .context("failed to query usage summary")?;

  let mut total_instances = 0i64;
  let mut total_duration_secs = 0.0f64;
  let mut total_cost_usd = 0.0f64;
  let mut by_region = Vec::new();

  for row in &rows {
    let instances: i64 = row.get("instances");
    let duration_secs: f64 = row.get("duration_secs");
    let cost_usd: f64 = row.get("cost_usd");

    total_instances += instances;
    total_duration_secs += duration_secs;
    total_cost_usd += cost_usd;

    by_region.push(RegionSummary {
      region: row.get("region"),
      instances,
      duration_secs,
      cost_usd,
    });
  }

  Ok(Json(SummaryResponse {
    total_instances,
    total_duration_secs,
    total_cost_usd,
    by_region,
  }))
}

// --- POST /usage/instances ---

#[derive(Debug, serde::Deserialize, JsonSchema, aide::OperationIo)]
pub struct InstancesRequest {
  pub limit: Option<i64>,
  pub cursor: Option<String>,
}

#[derive(Debug, serde::Serialize, JsonSchema)]
pub struct UsageInstance {
  pub id: String,
  pub status: String,
  pub region: String,
  pub vm_memory_mb: i32,
  pub cost_per_sec_usd: f64,
  pub created_at: DateTime<Utc>,
  pub started_at: Option<DateTime<Utc>>,
  pub stopped_at: Option<DateTime<Utc>>,
  pub duration_secs: i64,
  pub cost_usd: f64,
}

#[derive(Debug, serde::Serialize, JsonSchema, aide::OperationIo)]
pub struct InstancesResponse {
  pub instances: Vec<UsageInstance>,
  pub next_cursor: Option<String>,
}

pub async fn instances(
  State(system): State<context::Context>,
  Json(body): Json<InstancesRequest>,
) -> Result<Json<InstancesResponse>, ApiError> {
  let limit = body.limit.unwrap_or(20).min(100);

  let client = system
    .pool()
    .get()
    .await
    .context("failed to get db connection")?;

  let sql = "SELECT
               id,
               status,
               region,
               vm_memory_mb,
               cost_per_sec_usd,
               created_at,
               started_at,
               stopped_at,
               extract(epoch FROM (coalesce(stopped_at, now()) - coalesce(started_at,
                 created_at)))::bigint AS duration_secs,
               cost_per_sec_usd * extract(epoch FROM (coalesce(stopped_at, now()) -
                 coalesce(started_at, created_at))) AS cost_usd
             FROM
               rebound.instance
             WHERE ($1::text IS NULL
               OR id < $1)
              ORDER BY
                id DESC
             LIMIT $2";

  let rows = client
    .query(sql, &[&body.cursor, &limit])
    .await
    .context("failed to query usage instances")?;

  let instances: Vec<UsageInstance> = rows
    .iter()
    .map(|row| UsageInstance {
      id: row.get("id"),
      status: row.get("status"),
      region: row.get("region"),
      vm_memory_mb: row.get("vm_memory_mb"),
      cost_per_sec_usd: row.get("cost_per_sec_usd"),
      created_at: row.get("created_at"),
      started_at: row.get("started_at"),
      stopped_at: row.get("stopped_at"),
      duration_secs: row.get("duration_secs"),
      cost_usd: row.get("cost_usd"),
    })
    .collect();

  let next_cursor = if instances.len() == limit as usize {
    instances.last().map(|i| i.id.clone())
  } else {
    None
  };

  Ok(Json(InstancesResponse {
    instances,
    next_cursor,
  }))
}
