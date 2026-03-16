use anyhow::Context;
use axum::Json;
use axum::extract::State;
use schemars::JsonSchema;

use crate::error::ApiError;
use crate::router::context;
use crate::system::SystemApi;

#[derive(Debug, serde::Deserialize, JsonSchema, aide::OperationIo)]
pub struct StopRequest {
  pub id: String,
}

#[derive(Debug, serde::Serialize, JsonSchema, aide::OperationIo)]
pub struct StopResponse {
  pub id: String,
  pub status: String,
}

pub async fn stop(
  State(system): State<context::Context>,
  Json(body): Json<StopRequest>,
) -> Result<Json<StopResponse>, ApiError> {
  let client = system
    .pool()
    .get()
    .await
    .context("failed to get db connection")?;

  let row = client
    .query_opt(
      "SELECT id, status, fly_app_name, fly_machine_id, tailscale_device_id
       FROM rebound.instance
       WHERE id = $1",
      &[&body.id],
    )
    .await
    .context("failed to query instance")?;

  let row = row.ok_or_else(|| ApiError::not_found("instance not found"))?;

  let status: String = row.get("status");
  if status != "launching" && status != "running" {
    return Err(ApiError::bad_request(format!(
      "cannot stop instance in '{}' state",
      status
    )));
  }

  client
    .execute(
      "UPDATE rebound.instance SET status = 'stopping' WHERE id = $1",
      &[&body.id],
    )
    .await
    .context("failed to update instance status")?;

  let pool = system.pool().clone();
  let fly = system.fly().clone();
  let ts = system.tailscale().clone();
  let instance_id = body.id.clone();
  let app_name: String = row.get("fly_app_name");
  let machine_id: Option<String> = row.get("fly_machine_id");
  let device_id: Option<String> = row.get("tailscale_device_id");

  tokio::spawn(async move {
    if let Err(e) = teardown(
      &pool,
      &fly,
      &ts,
      &TeardownParams {
        instance_id: &instance_id,
        app_name: &app_name,
        machine_id: machine_id.as_deref(),
        device_id: device_id.as_deref(),
        stop_reason: "user_requested",
      },
    )
    .await
    {
      log::error!("teardown failed for instance {}: {:?}", instance_id, e);
    }
  });

  Ok(Json(StopResponse {
    id: body.id,
    status: "stopping".to_string(),
  }))
}

pub struct TeardownParams<'a> {
  pub instance_id: &'a str,
  pub app_name: &'a str,
  pub machine_id: Option<&'a str>,
  pub device_id: Option<&'a str>,
  pub stop_reason: &'a str,
}

pub async fn teardown(
  pool: &deadpool_postgres::Pool,
  fly: &fly_client::FlyClient,
  ts: &tailscale_client::TailscaleClient,
  params: &TeardownParams<'_>,
) -> anyhow::Result<()> {
  if let Some(mid) = params.machine_id {
    let _ = fly.stop_machine(params.app_name, mid).await;
  }

  let _ = fly.destroy_app(params.app_name, true).await;

  if let Some(did) = params.device_id {
    let _ = ts.delete_device(did).await;
  }

  let client = pool.get().await?;
  client
    .execute(
      "UPDATE rebound.instance
       SET status = 'stopped', stopped_at = now(), stop_reason = $2
       WHERE id = $1",
      &[&params.instance_id, &params.stop_reason],
    )
    .await?;

  log::info!(
    "instance {} torn down (reason: {})",
    params.instance_id,
    params.stop_reason
  );
  Ok(())
}
