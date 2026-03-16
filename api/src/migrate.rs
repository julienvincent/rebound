use anyhow::Result;
use refinery::config::{Config, ConfigDbType};
use refinery::embed_migrations;

use crate::config;
use crate::system::{System, SystemApi};

embed_migrations!("migrations");

pub async fn migrate(config: &config::Config, system: &System) -> Result<()> {
  system
    .pool()
    .get()
    .await?
    .simple_query("CREATE SCHEMA IF NOT EXISTS rebound")
    .await?;

  let mut conn = Config::new(ConfigDbType::Postgres)
    .set_db_user(&config.postgres.username)
    .set_db_pass(&config.postgres.password)
    .set_db_host(&config.postgres.host)
    .set_db_port(&config.postgres.port.to_string())
    .set_db_name(&config.postgres.database);

  tokio::task::spawn_blocking(move || -> Result<()> {
    migrations::runner()
      .set_migration_table_name("rebound.migrations")
      .run(&mut conn)?;
    Ok(())
  })
  .await??;

  Ok(())
}
