use std::collections::HashMap;

use anyhow::Context;
use axum::Json;
use axum::extract::State;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;

use crate::error::ApiError;
use crate::router::context;
use crate::system::SystemApi;

use super::compute_cost_per_sec;

#[derive(Debug, serde::Deserialize, JsonSchema, aide::OperationIo)]
pub struct CreateRequest {
  pub region: String,
  #[serde(default = "default_memory")]
  pub vm_memory_mb: i32,
  #[serde(default = "default_max_duration")]
  pub max_duration_secs: i32,
}

fn default_memory() -> i32 {
  512
}

fn default_max_duration() -> i32 {
  21600
}

#[derive(Debug, serde::Serialize, JsonSchema, aide::OperationIo)]
pub struct CreateResponse {
  pub id: String,
  pub status: String,
  pub region: String,
  pub vm_memory_mb: i32,
  pub fly_app_name: String,
  pub tailscale_node_name: String,
  pub created_at: DateTime<Utc>,
}

pub async fn create(
  State(system): State<context::Context>,
  Json(body): Json<CreateRequest>,
) -> Result<Json<CreateResponse>, ApiError> {
  let valid_regions: Vec<&str> = crate::routes::region::REGIONS
    .iter()
    .map(|r| r.code)
    .collect();
  if !valid_regions.contains(&body.region.as_str()) {
    return Err(ApiError::bad_request(format!(
      "invalid region: {}",
      body.region
    )));
  }

  let id = crate::kid::generate();
  let suffix = id[id.len() - 8..].to_ascii_lowercase();
  let fly_app_name = format!("rebound-{}", suffix);
  let tailscale_node_name = format!("rebound-exit-{}", suffix);
  let cost_per_sec = compute_cost_per_sec(body.vm_memory_mb);
  let exit_node_tag = system.exit_node_tag().to_string();

  let client = system
    .pool()
    .get()
    .await
    .context("failed to get db connection")?;

  let row = client
    .query_one(
      "INSERT INTO rebound.instance (id, status, region, vm_memory_mb, fly_app_name,
         tailscale_node_name, cost_per_sec_usd, max_duration_secs)
         VALUES ($1, 'launching', $2, $3, $4, $5, $6, $7)
       RETURNING
         id, status, region, vm_memory_mb, fly_app_name, tailscale_node_name, created_at",
      &[
        &id,
        &body.region,
        &body.vm_memory_mb,
        &fly_app_name,
        &tailscale_node_name,
        &cost_per_sec,
        &body.max_duration_secs,
      ],
    )
    .await
    .context("failed to insert instance")?;

  let response = CreateResponse {
    id: row.get("id"),
    status: row.get("status"),
    region: row.get("region"),
    vm_memory_mb: row.get("vm_memory_mb"),
    fly_app_name: row.get("fly_app_name"),
    tailscale_node_name: row.get("tailscale_node_name"),
    created_at: row.get("created_at"),
  };

  let pool = system.pool().clone();
  let fly = system.fly().clone();
  let ts = system.tailscale().clone();
  let params = LaunchParams {
    instance_id: response.id.clone(),
    region: body.region.clone(),
    vm_memory_mb: body.vm_memory_mb,
    app_name: fly_app_name.clone(),
    node_name: tailscale_node_name.clone(),
    exit_node_tag,
  };

  tokio::spawn(async move {
    if let Err(e) = launch_background(pool, fly, ts, &params).await {
      log::error!(
        "launch background task failed for {}: {:?}",
        params.instance_id,
        e
      );
    }
  });

  Ok(Json(response))
}

struct LaunchParams {
  instance_id: String,
  region: String,
  vm_memory_mb: i32,
  app_name: String,
  node_name: String,
  exit_node_tag: String,
}

async fn launch_background(
  pool: deadpool_postgres::Pool,
  fly: fly_client::FlyClient,
  ts: tailscale_client::TailscaleClient,
  params: &LaunchParams,
) -> anyhow::Result<()> {
  let result = launch_inner(&pool, &fly, &ts, params).await;

  if let Err(ref e) = result {
    log::error!("instance {} launch failed: {:?}", params.instance_id, e);

    let client = pool.get().await?;
    client
      .execute(
        "UPDATE
           rebound.instance
         SET
           status = 'failed',
           error = $1
         WHERE
           id = $2",
        &[&e.to_string(), &params.instance_id.as_str()],
      )
      .await?;

    // best-effort cleanup
    let _ = fly.destroy_app(&params.app_name, true).await;
  }

  result
}

async fn launch_inner(
  pool: &deadpool_postgres::Pool,
  fly: &fly_client::FlyClient,
  ts: &tailscale_client::TailscaleClient,
  params: &LaunchParams,
) -> anyhow::Result<()> {
  let auth_key_response = ts
    .create_auth_key(&tailscale_client::CreateKeyRequest {
      capabilities: tailscale_client::KeyCapabilities {
        devices: tailscale_client::DeviceCapabilities {
          create: tailscale_client::DeviceCreateCapabilities {
            reusable: false,
            ephemeral: true,
            preauthorized: true,
            tags: vec![params.exit_node_tag.clone()],
          },
        },
      },
      expiry_seconds: Some(600),
    })
    .await
    .context("failed to create tailscale auth key")?;

  {
    let client = pool.get().await?;
    client
      .execute(
        "UPDATE
           rebound.instance
         SET
           tailscale_auth_key = $1
         WHERE
           id = $2",
        &[&auth_key_response.key, &params.instance_id.as_str()],
      )
      .await?;
  }

  fly
    .create_app(&params.app_name, "personal")
    .await
    .context("failed to create fly app")?;

  let mut env = HashMap::new();
  env.insert("TS_AUTHKEY".to_string(), auth_key_response.key.clone());
  env.insert("TS_HOSTNAME".to_string(), params.node_name.clone());
  env.insert(
    "TS_EXTRA_ARGS".to_string(),
    "--advertise-exit-node".to_string(),
  );

  let machine = fly
    .create_machine(
      &params.app_name,
      &fly_client::CreateMachineRequest {
        name: Some(params.node_name.clone()),
        region: Some(params.region.clone()),
        config: fly_client::MachineConfig {
          image: "tailscale/tailscale:latest".to_string(),
          auto_destroy: Some(true),
          env: Some(env),
          guest: Some(fly_client::GuestConfig {
            cpu_kind: "shared".to_string(),
            cpus: 1,
            memory_mb: params.vm_memory_mb as u32,
          }),
          restart: Some(fly_client::RestartPolicy {
            policy: "always".to_string(),
            max_retries: None,
          }),
        },
      },
    )
    .await
    .context("failed to create fly machine")?;

  {
    let client = pool.get().await?;
    client
      .execute(
        "UPDATE
           rebound.instance
         SET
           fly_machine_id = $1
         WHERE
           id = $2",
        &[&machine.id, &params.instance_id.as_str()],
      )
      .await?;
  }

  fly
    .wait_for_machine(&params.app_name, &machine.id, "started", 60)
    .await
    .context("timed out waiting for machine to start")?;

  let mut device_id = None;
  for _ in 0..30 {
    if let Some(device) = ts
      .find_device_by_hostname(&params.node_name)
      .await
      .context("failed to query tailscale devices")?
    {
      device_id = Some(device.id);
      break;
    }
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
  }

  let device_id =
    device_id.ok_or_else(|| anyhow::anyhow!("timed out waiting for tailscale device to appear"))?;

  {
    let client = pool.get().await?;
    client
      .execute(
        "UPDATE
           rebound.instance
         SET
           status = 'running',
           tailscale_device_id = $1,
           started_at = now()
         WHERE
           id = $2",
        &[&device_id, &params.instance_id.as_str()],
      )
      .await?;
  }

  log::info!("instance {} is now running", params.instance_id);
  Ok(())
}
