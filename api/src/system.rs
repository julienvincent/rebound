use anyhow::Result;
use deadpool_postgres::Runtime;
use fly_client::FlyClient;
use std::sync::Arc;
use tailscale_client::TailscaleClient;
use tokio_postgres::NoTls;

use crate::config::Config;

pub trait SystemApi {
  fn pool(&self) -> &deadpool_postgres::Pool;
  fn fly(&self) -> &FlyClient;
  fn tailscale(&self) -> &TailscaleClient;
  fn exit_node_tag(&self) -> &str;
}

#[derive(Clone)]
pub struct System {
  db_pool: deadpool_postgres::Pool,
  fly_client: Arc<FlyClient>,
  tailscale_client: Arc<TailscaleClient>,
  exit_node_tag: String,
}

pub const DEFAULT_EXIT_NODE_TAG: &str = "tag:rebound-exit";

fn normalize_exit_node_tag(tag: &str) -> String {
  let trimmed = tag.trim();
  if trimmed.is_empty() {
    return DEFAULT_EXIT_NODE_TAG.to_string();
  }

  if trimmed.starts_with("tag:") {
    trimmed.to_string()
  } else {
    format!("tag:{trimmed}")
  }
}

impl System {
  pub async fn new(config: &Config) -> Result<Self> {
    let mut pg_config = deadpool_postgres::Config::new();
    pg_config.host = Some(config.postgres.host.clone());
    pg_config.port = Some(config.postgres.port as u16);
    pg_config.user = Some(config.postgres.username.clone());
    pg_config.password = Some(config.postgres.password.clone());
    pg_config.dbname = Some(config.postgres.database.clone());

    let db_pool = pg_config.create_pool(Some(Runtime::Tokio1), NoTls)?;

    let fly_client = Arc::new(FlyClient::new(&config.fly.api_token));
    let tailscale_client = Arc::new(TailscaleClient::new(
      &config.tailscale.api_key,
      &config.tailscale.tailnet,
    ));
    let exit_node_tag = config
      .tailscale
      .exit_node_tag
      .clone()
      .map(|tag| normalize_exit_node_tag(&tag))
      .unwrap_or_else(|| DEFAULT_EXIT_NODE_TAG.to_string());

    Ok(System {
      db_pool,
      fly_client,
      tailscale_client,
      exit_node_tag,
    })
  }
}

impl SystemApi for System {
  fn pool(&self) -> &deadpool_postgres::Pool {
    &self.db_pool
  }

  fn fly(&self) -> &FlyClient {
    &self.fly_client
  }

  fn tailscale(&self) -> &TailscaleClient {
    &self.tailscale_client
  }

  fn exit_node_tag(&self) -> &str {
    &self.exit_node_tag
  }
}
