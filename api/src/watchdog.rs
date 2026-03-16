use anyhow::Context;

use crate::routes::instance::TeardownParams;
use crate::system::SystemApi;

struct InstanceRow {
  id: String,
  fly_app_name: String,
  fly_machine_id: Option<String>,
  tailscale_device_id: Option<String>,
}

pub async fn run_cleanup(system: &impl SystemApi) -> anyhow::Result<Vec<String>> {
  let stale_instances = {
    let client = system
      .pool()
      .get()
      .await
      .context("failed to get db connection")?;

    let stale_launching = client
      .query(
        "SELECT id, fly_app_name, fly_machine_id, tailscale_device_id
         FROM rebound.instance
         WHERE status = 'launching'
           AND created_at < now() - interval '5 minutes'",
        &[],
      )
      .await
      .context("failed to query stale launching instances")?;

    let stale_stopping = client
      .query(
        "SELECT id, fly_app_name, fly_machine_id, tailscale_device_id
         FROM rebound.instance
         WHERE status = 'stopping'
           AND created_at < now() - interval '2 minutes'",
        &[],
      )
      .await
      .context("failed to query stale stopping instances")?;

    stale_launching
      .iter()
      .chain(stale_stopping.iter())
      .map(|row| InstanceRow {
        id: row.get("id"),
        fly_app_name: row.get("fly_app_name"),
        fly_machine_id: row.get("fly_machine_id"),
        tailscale_device_id: row.get("tailscale_device_id"),
      })
      .collect::<Vec<_>>()
  };

  let mut cleaned = Vec::new();

  for instance in &stale_instances {
    log::info!("watchdog: cleaning up stale instance {}", instance.id);

    if let Err(e) = crate::routes::instance::teardown(
      system.pool(),
      system.fly(),
      system.tailscale(),
      &TeardownParams {
        instance_id: &instance.id,
        app_name: &instance.fly_app_name,
        machine_id: instance.fly_machine_id.as_deref(),
        device_id: instance.tailscale_device_id.as_deref(),
        stop_reason: "watchdog_stale",
      },
    )
    .await
    {
      log::error!("watchdog: teardown failed for {}: {:?}", instance.id, e);

      if let Ok(client) = system.pool().get().await {
        let _ = client
          .execute(
            "UPDATE rebound.instance
             SET status = 'failed', error = $1, stopped_at = now(), stop_reason = 'watchdog_stale'
             WHERE id = $2",
            &[&e.to_string(), &instance.id],
          )
          .await;
      }
    }

    cleaned.push(instance.id.clone());
  }

  // Find running instances that have exceeded their max duration
  let expired_instances = {
    let client = system
      .pool()
      .get()
      .await
      .context("failed to get db connection")?;

    client
      .query(
        "SELECT id, fly_app_name, fly_machine_id, tailscale_device_id
         FROM rebound.instance
         WHERE status = 'running'
           AND started_at IS NOT NULL
           AND started_at + make_interval(secs => max_duration_secs::double precision) < now()",
        &[],
      )
      .await
      .context("failed to query expired instances")?
      .iter()
      .map(|row| InstanceRow {
        id: row.get("id"),
        fly_app_name: row.get("fly_app_name"),
        fly_machine_id: row.get("fly_machine_id"),
        tailscale_device_id: row.get("tailscale_device_id"),
      })
      .collect::<Vec<_>>()
  };

  for instance in &expired_instances {
    log::info!(
      "watchdog: instance {} exceeded max duration, tearing down",
      instance.id
    );

    if let Err(e) = crate::routes::instance::teardown(
      system.pool(),
      system.fly(),
      system.tailscale(),
      &TeardownParams {
        instance_id: &instance.id,
        app_name: &instance.fly_app_name,
        machine_id: instance.fly_machine_id.as_deref(),
        device_id: instance.tailscale_device_id.as_deref(),
        stop_reason: "max_duration_exceeded",
      },
    )
    .await
    {
      log::error!("watchdog: teardown failed for {}: {:?}", instance.id, e);
    }

    cleaned.push(instance.id.clone());
  }

  // Check running instances whose Fly app may have disappeared
  let running_instances = {
    let client = system
      .pool()
      .get()
      .await
      .context("failed to get db connection")?;

    client
      .query(
        "SELECT id, fly_app_name, fly_machine_id, tailscale_device_id
         FROM rebound.instance
         WHERE status = 'running'",
        &[],
      )
      .await
      .context("failed to query running instances")?
      .iter()
      .map(|row| InstanceRow {
        id: row.get("id"),
        fly_app_name: row.get("fly_app_name"),
        fly_machine_id: row.get("fly_machine_id"),
        tailscale_device_id: row.get("tailscale_device_id"),
      })
      .collect::<Vec<_>>()
  };

  for instance in &running_instances {
    let machines = system.fly().list_machines(&instance.fly_app_name).await;
    let is_alive = match machines {
      Ok(list) => !list.is_empty(),
      Err(_) => false,
    };

    if !is_alive {
      log::info!(
        "watchdog: running instance {} has no live machines, tearing down",
        instance.id
      );

      if let Err(e) = crate::routes::instance::teardown(
        system.pool(),
        system.fly(),
        system.tailscale(),
        &TeardownParams {
          instance_id: &instance.id,
          app_name: &instance.fly_app_name,
          machine_id: instance.fly_machine_id.as_deref(),
          device_id: instance.tailscale_device_id.as_deref(),
          stop_reason: "watchdog_orphaned",
        },
      )
      .await
      {
        log::error!("watchdog: teardown failed for {}: {:?}", instance.id, e);
      }

      cleaned.push(instance.id.clone());
    }
  }

  Ok(cleaned)
}

pub fn start_watchdog_loop(system: impl SystemApi + Send + Sync + 'static, interval_secs: u64) {
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
    loop {
      interval.tick().await;
      log::info!("watchdog: running cleanup pass");
      match run_cleanup(&system).await {
        Ok(cleaned) if !cleaned.is_empty() => {
          log::info!("watchdog: cleaned {} instances", cleaned.len());
        }
        Ok(_) => {}
        Err(e) => {
          log::error!("watchdog: cleanup pass failed: {:?}", e);
        }
      }
    }
  });
}
