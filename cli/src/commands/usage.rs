use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::OutputFormat;
use crate::client::Client;
use crate::output;

#[derive(Debug, clap::Args)]
pub struct UsageArgs {
  #[command(subcommand)]
  pub command: UsageCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum UsageCommand {
  /// Get a cost/duration summary, optionally filtered by time range
  Summary {
    /// Start of time range (RFC3339)
    #[arg(long)]
    from: Option<DateTime<Utc>>,

    /// End of time range (RFC3339)
    #[arg(long)]
    to: Option<DateTime<Utc>>,
  },
  /// List instance usage records
  Instances {
    /// Maximum number of results
    #[arg(long)]
    limit: Option<i64>,

    /// Pagination cursor
    #[arg(long)]
    cursor: Option<String>,
  },
}

fn fmt_optional_dt(val: &Option<DateTime<Utc>>) -> String {
  match val {
    Some(dt) => dt.to_string(),
    None => "-".to_string(),
  }
}

// --- Request/Response types ---

#[derive(Debug, Serialize)]
struct SummaryRequest {
  #[serde(skip_serializing_if = "Option::is_none")]
  from: Option<DateTime<Utc>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  to: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SummaryResponse {
  pub total_instances: i64,
  pub total_duration_secs: f64,
  pub total_cost_usd: f64,
  pub by_region: Vec<RegionSummary>,
}

#[derive(Debug, Deserialize, Serialize, Tabled)]
pub struct RegionSummary {
  #[tabled(rename = "REGION")]
  pub region: String,
  #[tabled(rename = "INSTANCES")]
  pub instances: i64,
  #[tabled(rename = "DURATION (s)")]
  pub duration_secs: f64,
  #[tabled(rename = "COST USD")]
  pub cost_usd: f64,
}

#[derive(Debug, Serialize)]
struct InstancesRequest {
  #[serde(skip_serializing_if = "Option::is_none")]
  limit: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct InstancesResponse {
  instances: Vec<UsageInstance>,
  next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Tabled)]
pub struct UsageInstance {
  #[tabled(rename = "ID")]
  pub id: String,
  #[tabled(rename = "STATUS")]
  pub status: String,
  #[tabled(rename = "REGION")]
  pub region: String,
  #[tabled(rename = "MEMORY MB")]
  pub vm_memory_mb: i32,
  #[tabled(rename = "COST/SEC")]
  pub cost_per_sec_usd: f64,
  #[tabled(rename = "CREATED AT")]
  pub created_at: DateTime<Utc>,
  #[tabled(rename = "STARTED AT", display = "fmt_optional_dt")]
  pub started_at: Option<DateTime<Utc>>,
  #[tabled(rename = "STOPPED AT", display = "fmt_optional_dt")]
  pub stopped_at: Option<DateTime<Utc>>,
  #[tabled(rename = "DURATION")]
  pub duration_secs: i64,
  #[tabled(rename = "COST USD")]
  pub cost_usd: f64,
}

// --- Command dispatch ---

pub fn run(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  args: UsageArgs,
  output: &OutputFormat,
) -> Result<()> {
  match args.command {
    UsageCommand::Summary { from, to } => summary(client, rt, from, to, output),
    UsageCommand::Instances { limit, cursor } => instances(client, rt, limit, cursor, output),
  }
}

fn summary(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  from: Option<DateTime<Utc>>,
  to: Option<DateTime<Utc>>,
  output: &OutputFormat,
) -> Result<()> {
  let response: SummaryResponse =
    rt.block_on(client.post("/usage/summary", &SummaryRequest { from, to }))?;

  match output {
    OutputFormat::Json => {
      println!("{}", serde_json::to_string_pretty(&response)?);
      Ok(())
    }
    OutputFormat::Table => {
      println!(
        "Total: {} instances, {:.0}s duration, ${:.4} USD",
        response.total_instances, response.total_duration_secs, response.total_cost_usd
      );
      println!();
      output::render_list(response.by_region, output, "No usage data")
    }
  }
}

fn instances(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  limit: Option<i64>,
  cursor: Option<String>,
  output: &OutputFormat,
) -> Result<()> {
  let response: InstancesResponse =
    rt.block_on(client.post("/usage/instances", &InstancesRequest { limit, cursor }))?;

  if let Some(ref next_cursor) = response.next_cursor {
    log::info!("Next cursor: {next_cursor}");
  }

  match output {
    OutputFormat::Json => {
      println!("{}", serde_json::to_string_pretty(&response)?);
      Ok(())
    }
    OutputFormat::Table => output::render_list(response.instances, output, "No usage instances"),
  }
}
