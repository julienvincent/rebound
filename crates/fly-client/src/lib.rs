use std::collections::HashMap;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("HTTP request failed: {0}")]
  Request(#[from] reqwest::Error),

  #[error("Fly API error ({status}): {message}")]
  Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct FlyClient {
  client: Client,
  base_url: String,
  token: String,
}

impl FlyClient {
  pub fn new(token: impl Into<String>) -> Self {
    Self {
      client: Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build reqwest client"),
      base_url: "https://api.machines.dev/v1".to_string(),
      token: token.into(),
    }
  }

  async fn check_response(&self, response: reqwest::Response) -> Result<reqwest::Response> {
    if response.status().is_success() {
      Ok(response)
    } else {
      let status = response.status().as_u16();
      let message = response
        .text()
        .await
        .unwrap_or_else(|_| "unknown error".to_string());
      Err(Error::Api { status, message })
    }
  }

  pub async fn create_app(&self, name: &str, org_slug: &str) -> Result<CreateAppResponse> {
    let response = self
      .client
      .post(format!("{}/apps", self.base_url))
      .bearer_auth(&self.token)
      .json(&CreateAppRequest {
        app_name: name.to_string(),
        org_slug: org_slug.to_string(),
      })
      .send()
      .await?;

    let response = self.check_response(response).await?;
    Ok(response.json().await?)
  }

  pub async fn destroy_app(&self, name: &str, force: bool) -> Result<()> {
    let mut url = format!("{}/apps/{}", self.base_url, name);
    if force {
      url.push_str("?force=true");
    }

    let response = self
      .client
      .delete(&url)
      .bearer_auth(&self.token)
      .send()
      .await?;

    self.check_response(response).await?;
    Ok(())
  }

  pub async fn create_machine(&self, app: &str, request: &CreateMachineRequest) -> Result<Machine> {
    let response = self
      .client
      .post(format!("{}/apps/{}/machines", self.base_url, app))
      .bearer_auth(&self.token)
      .json(request)
      .send()
      .await?;

    let response = self.check_response(response).await?;
    Ok(response.json().await?)
  }

  pub async fn get_machine(&self, app: &str, machine_id: &str) -> Result<Machine> {
    let response = self
      .client
      .get(format!(
        "{}/apps/{}/machines/{}",
        self.base_url, app, machine_id
      ))
      .bearer_auth(&self.token)
      .send()
      .await?;

    let response = self.check_response(response).await?;
    Ok(response.json().await?)
  }

  pub async fn list_machines(&self, app: &str) -> Result<Vec<Machine>> {
    let response = self
      .client
      .get(format!("{}/apps/{}/machines", self.base_url, app))
      .bearer_auth(&self.token)
      .send()
      .await?;

    let response = self.check_response(response).await?;
    Ok(response.json().await?)
  }

  pub async fn stop_machine(&self, app: &str, machine_id: &str) -> Result<()> {
    let response = self
      .client
      .post(format!(
        "{}/apps/{}/machines/{}/stop",
        self.base_url, app, machine_id
      ))
      .bearer_auth(&self.token)
      .send()
      .await?;

    self.check_response(response).await?;
    Ok(())
  }

  pub async fn delete_machine(&self, app: &str, machine_id: &str, force: bool) -> Result<()> {
    let mut url = format!("{}/apps/{}/machines/{}", self.base_url, app, machine_id);
    if force {
      url.push_str("?force=true");
    }

    let response = self
      .client
      .delete(&url)
      .bearer_auth(&self.token)
      .send()
      .await?;

    self.check_response(response).await?;
    Ok(())
  }

  pub async fn wait_for_machine(
    &self,
    app: &str,
    machine_id: &str,
    state: &str,
    timeout_secs: u64,
  ) -> Result<()> {
    let response = self
      .client
      .get(format!(
        "{}/apps/{}/machines/{}/wait?state={}&timeout={}",
        self.base_url, app, machine_id, state, timeout_secs
      ))
      .bearer_auth(&self.token)
      .timeout(Duration::from_secs(timeout_secs + 10))
      .send()
      .await?;

    self.check_response(response).await?;
    Ok(())
  }
}

#[derive(Debug, Serialize)]
struct CreateAppRequest {
  app_name: String,
  org_slug: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAppResponse {
  pub id: String,
  pub created_at: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateMachineRequest {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub region: Option<String>,
  pub config: MachineConfig,
}

#[derive(Debug, Serialize)]
pub struct MachineConfig {
  pub image: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auto_destroy: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<HashMap<String, String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub guest: Option<GuestConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub restart: Option<RestartPolicy>,
}

#[derive(Debug, Serialize)]
pub struct GuestConfig {
  pub cpu_kind: String,
  pub cpus: u32,
  pub memory_mb: u32,
}

#[derive(Debug, Serialize)]
pub struct RestartPolicy {
  pub policy: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_retries: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Machine {
  pub id: String,
  #[serde(default)]
  pub name: String,
  pub state: String,
  #[serde(default)]
  pub region: String,
  #[serde(default)]
  pub instance_id: Option<String>,
  #[serde(default)]
  pub private_ip: Option<String>,
  #[serde(default)]
  pub created_at: Option<String>,
  #[serde(default)]
  pub updated_at: Option<String>,
}
