pub mod client;
pub mod commands;
pub mod error;
pub mod output;

use anyhow::Result;
use clap::ValueEnum;

#[derive(Debug, clap::Args)]
pub struct GlobalOpts {
  /// Rebound server URL
  #[arg(long, global = true)]
  pub host: Option<String>,

  #[arg(long, global = true)]
  pub log_level: Option<log::LevelFilter>,

  #[arg(long, short = 'o', global = true, default_value = "table")]
  pub output: OutputFormat,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
  #[default]
  Table,
  Json,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
  /// Manage regions
  Region(commands::region::RegionArgs),
  /// Manage instances
  Instance(commands::instance::InstanceArgs),
  /// Query usage data
  Usage(commands::usage::UsageArgs),
  /// Health check
  Livez,
  /// Generate shell completions
  Completions {
    #[arg(value_enum)]
    shell: clap_complete::Shell,
  },
}

pub fn run(command: Command, opts: &GlobalOpts) -> Result<()> {
  let host = opts
    .host
    .as_deref()
    .ok_or_else(|| anyhow::anyhow!("--host is required"))?;

  let rt = tokio::runtime::Runtime::new()?;
  let client = client::Client::new(host);

  match command {
    Command::Region(args) => commands::region::run(&client, &rt, args, &opts.output),
    Command::Instance(args) => commands::instance::run(&client, &rt, args, &opts.output),
    Command::Usage(args) => commands::usage::run(&client, &rt, args, &opts.output),
    Command::Livez => commands::livez::run(&client, &rt),
    Command::Completions { .. } => unreachable!(),
  }
}
