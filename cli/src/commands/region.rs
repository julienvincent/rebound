use anyhow::Result;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::OutputFormat;
use crate::client::Client;
use crate::output;

#[derive(Debug, clap::Args)]
pub struct RegionArgs {
  #[command(subcommand)]
  pub command: RegionCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum RegionCommand {
  /// List available regions
  List,
}

#[derive(Debug, Deserialize)]
pub struct ListRegionsResponse {
  pub regions: Vec<Region>,
}

#[derive(Debug, Deserialize, Serialize, Tabled)]
pub struct Region {
  #[tabled(rename = "CODE")]
  pub code: String,
  #[tabled(rename = "NAME")]
  pub name: String,
}

pub fn run(
  client: &Client,
  rt: &tokio::runtime::Runtime,
  args: RegionArgs,
  output: &OutputFormat,
) -> Result<()> {
  match args.command {
    RegionCommand::List => list(client, rt, output),
  }
}

fn list(client: &Client, rt: &tokio::runtime::Runtime, output: &OutputFormat) -> Result<()> {
  let response: ListRegionsResponse =
    rt.block_on(client.post("/region/list", &serde_json::json!({})))?;

  output::render_list(response.regions, output, "No regions found")
}
