use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::OutputFormat;
use crate::client::Client;
use crate::output;

#[derive(Debug, clap::Args)]
pub struct InstanceArgs {
  #[command(subcommand)]
  pub command: InstanceCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum InstanceCommand {
  /// Create a new exit node instance
  Create {
    /// Region code (e.g. syd, lhr, iad)
    #[arg(long)]
    region: Option<String>,

    /// VM memory in MB
    #[arg(long)]
    vm_memory_mb: Option<i32>,

    /// Maximum instance duration in seconds
    #[arg(long)]
    max_duration_secs: Option<i32>,

    /// Return immediately without waiting for the instance to start
    #[arg(long = "async")]
    is_async: bool,
  },
  /// Get details of an instance
  Get {
    /// Instance ID
    #[arg(long)]
    id: String,
  },
  /// List instances
  List {
    /// Filter by status (launching, running, stopping, stopped, failed)
    #[arg(long)]
    status: Option<String>,

    /// Filter by region code
    #[arg(long)]
    region: Option<String>,

    /// Maximum number of results
    #[arg(long)]
    limit: Option<i64>,

    /// Pagination cursor
    #[arg(long)]
    cursor: Option<String>,
  },
  /// Stop a running instance
  Stop {
    /// Instance ID
    #[arg(long)]
    id: Option<String>,

    /// Return immediately without waiting for the instance to stop
    #[arg(long = "async")]
    is_async: bool,
  },
}

fn fmt_optional_dt(val: &Option<DateTime<Utc>>) -> String {
  match val {
    Some(dt) => dt.to_string(),
    None => "-".to_string(),
  }
}

fn fmt_optional_str(val: &Option<String>) -> String {
  match val {
    Some(s) => s.clone(),
    None => "-".to_string(),
  }
}

// --- Request/Response types ---

#[derive(Debug, Serialize)]
struct CreateRequest {
  region: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  vm_memory_mb: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  max_duration_secs: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Tabled)]
pub struct CreateResponse {
  #[tabled(rename = "ID")]
  pub id: String,
  #[tabled(rename = "STATUS")]
  pub status: String,
  #[tabled(rename = "FLY APP")]
  pub fly_app_name: String,
  #[tabled(skip)]
  pub region: String,
  #[tabled(skip)]
  pub vm_memory_mb: i32,
  #[tabled(skip)]
  pub tailscale_node_name: String,
  #[tabled(skip)]
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Tabled)]
struct ListRow {
  #[tabled(rename = "ID")]
  id: String,
  #[tabled(rename = "STATUS")]
  status: String,
  #[tabled(rename = "REGION")]
  region: String,
  #[tabled(rename = "FLY APP")]
  fly_app_name: String,
  #[tabled(rename = "DURATION")]
  duration_secs: i64,
  #[tabled(rename = "COST USD")]
  cost_usd: String,
}

#[derive(Debug, Serialize, Tabled)]
struct StopRow {
  #[tabled(rename = "ID")]
  id: String,
  #[tabled(rename = "STATUS")]
  status: String,
  #[tabled(rename = "FLY APP")]
  fly_app_name: String,
}

impl From<Instance> for StopRow {
  fn from(instance: Instance) -> Self {
    Self {
      id: instance.id,
      status: instance.status,
      fly_app_name: instance.fly_app_name,
    }
  }
}

impl From<Instance> for ListRow {
  fn from(instance: Instance) -> Self {
    Self {
      id: instance.id,
      status: instance.status,
      region: instance.region,
      fly_app_name: instance.fly_app_name,
      duration_secs: instance.duration_secs,
      cost_usd: format!("{:.5}", instance.cost_usd),
    }
  }
}

#[derive(Debug, Serialize)]
struct GetRequest {
  id: String,
}

#[derive(Debug, Deserialize, Serialize, Tabled)]
pub struct Instance {
  #[tabled(rename = "ID")]
  pub id: String,
  #[tabled(rename = "STATUS")]
  pub status: String,
  #[tabled(rename = "REGION")]
  pub region: String,
  #[tabled(rename = "MEMORY MB")]
  pub vm_memory_mb: i32,
  #[tabled(rename = "FLY APP")]
  pub fly_app_name: String,
  #[tabled(rename = "TS NODE")]
  pub tailscale_node_name: String,
  #[tabled(rename = "COST/SEC")]
  pub cost_per_sec_usd: f64,
  #[tabled(rename = "MAX DUR")]
  pub max_duration_secs: i32,
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
  #[tabled(skip)]
  pub fly_machine_id: Option<String>,
  #[tabled(skip)]
  pub tailscale_device_id: Option<String>,
  #[tabled(rename = "STOP REASON", display = "fmt_optional_str")]
  pub stop_reason: Option<String>,
  #[tabled(rename = "ERROR", display = "fmt_optional_str")]
  pub error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ListRequest {
  #[serde(skip_serializing_if = "Option::is_none")]
  status: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  region: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  limit: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ListResponse {
  instances: Vec<Instance>,
  next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
struct StopRequest {
  id: String,
}

#[derive(Debug, Deserialize, Serialize, Tabled)]
pub struct StopResponse {
  #[tabled(rename = "ID")]
  pub id: String,
  #[tabled(rename = "STATUS")]
  pub status: String,
}

// --- Command dispatch ---

pub fn run(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  args: InstanceArgs,
  output: &OutputFormat,
) -> Result<()> {
  match args.command {
    InstanceCommand::Create {
      region,
      vm_memory_mb,
      max_duration_secs,
      is_async,
    } => create(
      client,
      rt,
      region,
      vm_memory_mb,
      max_duration_secs,
      is_async,
      output,
    ),
    InstanceCommand::Get { id } => get(client, rt, id, output),
    InstanceCommand::List {
      status,
      region,
      limit,
      cursor,
    } => list(client, rt, status, region, limit, cursor, output),
    InstanceCommand::Stop { id, is_async } => stop(client, rt, id, is_async, output),
  }
}

fn create(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  region: Option<String>,
  vm_memory_mb: Option<i32>,
  max_duration_secs: Option<i32>,
  is_async: bool,
  output: &OutputFormat,
) -> Result<()> {
  let region = match region {
    Some(r) => r,
    None => prompt_region(client, rt)?,
  };

  let vm_memory_mb = Some(match vm_memory_mb {
    Some(v) => v,
    None => prompt_vm_memory_mb()?,
  });

  let max_duration_secs = Some(match max_duration_secs {
    Some(v) => v,
    None => prompt_max_duration_secs()?,
  });

  let response: CreateResponse = rt.block_on(client.post(
    "/instance/create",
    &CreateRequest {
      region,
      vm_memory_mb,
      max_duration_secs,
    },
  ))?;

  if is_async {
    return output::render_one(response, output);
  }

  let id = response.id.clone();
  eprint!("Launching instance {}...", id);

  rt.block_on(async {
    loop {
      tokio::time::sleep(std::time::Duration::from_secs(2)).await;
      let instance: Instance = client
        .post("/instance/get", &GetRequest { id: id.clone() })
        .await?;

      match instance.status.as_str() {
        "launching" => {
          eprint!(".");
        }
        "running" => {
          eprintln!(" running");
          let summary = CreateResponse {
            id: instance.id,
            status: instance.status,
            fly_app_name: instance.fly_app_name,
            region: instance.region,
            vm_memory_mb: instance.vm_memory_mb,
            tailscale_node_name: instance.tailscale_node_name,
            created_at: instance.created_at,
          };
          output::render_one(summary, output)?;
          return Ok(());
        }
        status => {
          eprintln!(" {status}");
          let err_msg = instance.error.as_deref().unwrap_or("unknown error");
          anyhow::bail!("Instance failed to start: {err_msg}");
        }
      }
    }
  })
}

fn prompt_region(client: &Client, rt: &tokio::runtime::Runtime) -> Result<String> {
  let response: super::region::ListRegionsResponse =
    rt.block_on(client.post("/region/list", &serde_json::json!({})))?;

  let options: Vec<String> = response
    .regions
    .iter()
    .map(|r| format!("{} ({})", r.code, r.name))
    .collect();

  let selection = inquire::Select::new("Select a region:", options)
    .prompt()
    .context("Region selection cancelled")?;

  let code = selection.split(' ').next().unwrap().to_string();
  Ok(code)
}

fn prompt_vm_memory_mb() -> Result<i32> {
  let options = [256, 512, 1024, 2048];
  let display: Vec<String> = options.iter().map(|v| format!("{v} MB")).collect();

  let selection = inquire::Select::new("VM memory:", display)
    .with_starting_cursor(1)
    .prompt()
    .context("VM memory selection cancelled")?;

  let value: i32 = selection.split(' ').next().unwrap().parse()?;
  Ok(value)
}

fn prompt_max_duration_secs() -> Result<i32> {
  let input: String = inquire::Text::new("Max duration (seconds):")
    .with_default("21600")
    .prompt()
    .context("Max duration input cancelled")?;

  let value: i32 = input.parse().context("Invalid number")?;
  Ok(value)
}

fn get(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  id: String,
  output: &OutputFormat,
) -> Result<()> {
  let response: Instance = rt.block_on(client.post("/instance/get", &GetRequest { id }))?;

  output::render_one(response, output)
}

fn list(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  status: Option<String>,
  region: Option<String>,
  limit: Option<i64>,
  cursor: Option<String>,
  output: &OutputFormat,
) -> Result<()> {
  let status = status.or_else(|| Some("running".to_string()));

  let response: ListResponse = rt.block_on(client.post(
    "/instance/list",
    &ListRequest {
      status,
      region,
      limit,
      cursor,
    },
  ))?;

  if let Some(ref next_cursor) = response.next_cursor {
    log::info!("Next cursor: {next_cursor}");
  }

  match output {
    OutputFormat::Json => {
      println!("{}", serde_json::to_string_pretty(&response)?);
      Ok(())
    }
    OutputFormat::Table => {
      let rows: Vec<ListRow> = response.instances.into_iter().map(ListRow::from).collect();
      output::render_list(rows, output, "No instances found")
    }
  }
}

fn stop(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  id: Option<String>,
  is_async: bool,
  output: &OutputFormat,
) -> Result<()> {
  let id = match id {
    Some(id) => id,
    None => prompt_stop_instance(client, rt)?,
  };

  let response: StopResponse =
    rt.block_on(client.post("/instance/stop", &StopRequest { id: id.clone() }))?;

  if is_async {
    return output::render_one(response, output);
  }

  eprint!("Stopping instance {}...", id);

  rt.block_on(async {
    loop {
      tokio::time::sleep(std::time::Duration::from_secs(2)).await;
      let instance: Instance = client
        .post("/instance/get", &GetRequest { id: id.clone() })
        .await?;

      match instance.status.as_str() {
        "stopping" => {
          eprint!(".");
        }
        "stopped" => {
          eprintln!(" stopped");
          return output::render_one(StopRow::from(instance), output);
        }
        status => {
          eprintln!(" {status}");
          let err_msg = instance.error.as_deref().unwrap_or("unknown error");
          anyhow::bail!("Instance failed to stop cleanly: {err_msg}");
        }
      }
    }
  })?;

  Ok(())
}

fn prompt_stop_instance(client: &Client, rt: &tokio::runtime::Runtime) -> Result<String> {
  let response: ListResponse = rt.block_on(client.post(
    "/instance/list",
    &ListRequest {
      status: Some("running".to_string()),
      region: None,
      limit: Some(100),
      cursor: None,
    },
  ))?;

  anyhow::ensure!(!response.instances.is_empty(), "No running instances found");

  let options: Vec<String> = response
    .instances
    .iter()
    .map(|instance| {
      format!(
        "{} | {} | {} | {:.5}",
        instance.id, instance.region, instance.fly_app_name, instance.cost_usd
      )
    })
    .collect();

  let selection = inquire::Select::new("Select an instance to stop:", options)
    .prompt()
    .context("Instance selection cancelled")?;

  Ok(
    selection
      .split(" | ")
      .next()
      .unwrap_or_default()
      .to_string(),
  )
}
