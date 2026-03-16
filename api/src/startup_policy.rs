use anyhow::{Result, bail};
use serde_json::Value;
use tailscale_client::TailscaleClient;

const REQUIRED_TAG_OWNER: &str = "autogroup:owner";

pub async fn check_tailscale_policy(ts: &TailscaleClient, tag: &str) -> Result<()> {
  let policy = ts.get_acl().await?;
  let mut missing = Vec::new();

  if !has_tag_owner(&policy, tag, REQUIRED_TAG_OWNER) {
    missing.push(format!(
      "missing tagOwners[{tag:?}] containing {REQUIRED_TAG_OWNER:?}"
    ));
  }

  if !has_exit_node_auto_approver(&policy, tag) {
    missing.push(format!("missing autoApprovers.exitNode entry for {tag:?}"));
  }

  if missing.is_empty() {
    log::info!("tailscale policy validated for exit node tag {}", tag);
    return Ok(());
  }

  log::error!("tailscale policy is missing required settings for {}:", tag);
  for issue in &missing {
    log::error!("- {}", issue);
  }

  bail!("tailscale policy validation failed")
}

fn has_tag_owner(policy: &Value, tag: &str, owner: &str) -> bool {
  policy
    .get("tagOwners")
    .and_then(Value::as_object)
    .and_then(|owners| owners.get(tag))
    .and_then(Value::as_array)
    .is_some_and(|entries| entries.iter().any(|entry| entry.as_str() == Some(owner)))
}

fn has_exit_node_auto_approver(policy: &Value, tag: &str) -> bool {
  policy
    .get("autoApprovers")
    .and_then(Value::as_object)
    .and_then(|auto| auto.get("exitNode"))
    .and_then(Value::as_array)
    .is_some_and(|entries| entries.iter().any(|entry| entry.as_str() == Some(tag)))
}
