#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct PostgresConfig {
  pub host: String,
  pub port: u64,
  pub database: String,
  pub username: String,
  pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct FlyConfig {
  pub api_token: String,
  pub org: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TailscaleConfig {
  pub api_key: String,
  pub tailnet: String,
  pub exit_node_tag: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Config {
  pub postgres: PostgresConfig,
  pub fly: FlyConfig,
  pub tailscale: TailscaleConfig,
  pub port: Option<u16>,
  pub watchdog_interval_secs: Option<u64>,
}
