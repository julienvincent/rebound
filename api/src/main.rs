use clap::{Parser, Subcommand};
use std::fs;

use router::build_router;
use system::{System, SystemApi};

mod config;
mod error;
mod kid;
mod migrate;
mod router;
mod routes;
mod startup_policy;
mod system;
mod watchdog;

#[derive(Parser, Debug)]
#[command(name = "rebound", version, about = "Ephemeral Tailscale exit node API")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
  Serve {
    #[arg(long)]
    config: String,
  },
  Migrate {
    #[arg(long)]
    config: String,
  },
  GenOpenapi,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
  env_logger::builder()
    .filter_level(log::LevelFilter::Info)
    .init();

  let cli = Cli::parse();

  match cli.command {
    Command::Serve {
      config: config_path,
    } => {
      let config_contents = fs::read_to_string(&config_path).expect("failed to read config file");
      let config: config::Config =
        serde_json::from_str(&config_contents).expect("invalid config JSON");

      let system = System::new(&config).await.unwrap();
      startup_policy::check_tailscale_policy(system.tailscale(), system.exit_node_tag())
        .await
        .unwrap_or_else(|err| {
          log::error!("refusing to start: {err}");
          std::process::exit(1);
        });

      let watchdog_interval = config.watchdog_interval_secs.unwrap_or(300);
      watchdog::start_watchdog_loop(system.clone(), watchdog_interval);

      let app = build_router(system);
      let port = config.port.unwrap_or(3000);
      let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

      log::info!("rebound listening on port {}", port);
      axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    }
    Command::Migrate {
      config: config_path,
    } => {
      let config_contents = fs::read_to_string(&config_path).expect("failed to read config file");
      let config: config::Config =
        serde_json::from_str(&config_contents).expect("invalid config JSON");

      let system = System::new(&config).await.unwrap();
      migrate::migrate(&config, &system).await.unwrap();
      log::info!("migrations complete");
    }
    Command::GenOpenapi => {
      log::set_max_level(log::LevelFilter::Off);

      struct MockSystem;

      impl SystemApi for MockSystem {
        fn pool(&self) -> &deadpool_postgres::Pool {
          unimplemented!()
        }
        fn fly(&self) -> &fly_client::FlyClient {
          unimplemented!()
        }
        fn tailscale(&self) -> &tailscale_client::TailscaleClient {
          unimplemented!()
        }
        fn exit_node_tag(&self) -> &str {
          unimplemented!()
        }
      }

      let router = build_router(MockSystem);
      let mut api = aide::openapi::OpenApi::default();
      let _ = router.finish_api(&mut api);
      println!("{}", serde_json::to_string_pretty(&api).unwrap());
    }
  }
}
