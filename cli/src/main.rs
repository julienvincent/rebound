use anyhow::Result;
use clap::{CommandFactory, Parser};

#[derive(Parser, Debug)]
#[command(name = "rebound-cli")]
#[command(about = "CLI for the Rebound ephemeral exit node API")]
struct Cli {
  #[clap(flatten)]
  global_opts: rebound_cli::GlobalOpts,

  #[command(subcommand)]
  command: rebound_cli::Command,
}

fn main() {
  let cli = Cli::parse();

  let mut log_builder = env_logger::builder();
  log_builder
    .format_timestamp(None)
    .format_target(false)
    .filter_level(cli.global_opts.log_level.unwrap_or(log::LevelFilter::Info));
  log_builder.init();

  if let Err(err) = run(cli) {
    eprintln!("Error: {err:#}");
    std::process::exit(1);
  }
}

fn run(cli: Cli) -> Result<()> {
  match cli.command {
    rebound_cli::Command::Completions { shell } => {
      clap_complete::generate(
        shell,
        &mut Cli::command(),
        "rebound-cli",
        &mut std::io::stdout(),
      );
      Ok(())
    }
    cmd => rebound_cli::run(cmd, &cli.global_opts),
  }
}
