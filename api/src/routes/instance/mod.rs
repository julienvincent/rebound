mod create;
mod get;
mod list;
mod stop;

pub use create::create;
pub use get::get;
pub use list::list;
pub use stop::{TeardownParams, stop, teardown};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use tokio_postgres::Row;

const CPU_COST_PER_SEC: f64 = 1.94 / (30.0 * 24.0 * 3600.0);
const RAM_COST_PER_256MB_PER_SEC: f64 = 2.36 / (30.0 * 24.0 * 3600.0);

pub fn compute_cost_per_sec(vm_memory_mb: i32) -> f64 {
  let ram_units = vm_memory_mb as f64 / 256.0;
  CPU_COST_PER_SEC + (ram_units * RAM_COST_PER_256MB_PER_SEC)
}

fn compute_instance_cost(
  cost_per_sec: f64,
  started_at: Option<DateTime<Utc>>,
  stopped_at: Option<DateTime<Utc>>,
) -> (f64, i64) {
  let start = match started_at {
    Some(t) => t,
    None => return (0.0, 0),
  };
  let end = stopped_at.unwrap_or_else(Utc::now);
  let duration_secs = (end - start).num_seconds().max(0);
  let cost = cost_per_sec * duration_secs as f64;
  (cost, duration_secs)
}

#[derive(Debug, serde::Serialize, JsonSchema)]
pub struct Instance {
  pub id: String,
  pub status: String,
  pub region: String,
  pub vm_memory_mb: i32,
  pub fly_app_name: String,
  pub fly_machine_id: Option<String>,
  pub tailscale_device_id: Option<String>,
  pub tailscale_node_name: String,
  pub cost_per_sec_usd: f64,
  pub max_duration_secs: i32,
  pub created_at: DateTime<Utc>,
  pub started_at: Option<DateTime<Utc>>,
  pub stopped_at: Option<DateTime<Utc>>,
  pub stop_reason: Option<String>,
  pub error: Option<String>,
  pub cost_usd: f64,
  pub duration_secs: i64,
}

pub fn row_to_instance(row: &Row) -> Instance {
  let cost_per_sec: f64 = row.get("cost_per_sec_usd");
  let started_at: Option<DateTime<Utc>> = row.get("started_at");
  let stopped_at: Option<DateTime<Utc>> = row.get("stopped_at");
  let (cost_usd, duration_secs) = compute_instance_cost(cost_per_sec, started_at, stopped_at);

  Instance {
    id: row.get("id"),
    status: row.get("status"),
    region: row.get("region"),
    vm_memory_mb: row.get("vm_memory_mb"),
    fly_app_name: row.get("fly_app_name"),
    fly_machine_id: row.get("fly_machine_id"),
    tailscale_device_id: row.get("tailscale_device_id"),
    tailscale_node_name: row.get("tailscale_node_name"),
    cost_per_sec_usd: cost_per_sec,
    max_duration_secs: row.get("max_duration_secs"),
    created_at: row.get("created_at"),
    started_at,
    stopped_at,
    stop_reason: row.get("stop_reason"),
    error: row.get("error"),
    cost_usd,
    duration_secs,
  }
}
