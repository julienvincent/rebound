use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("HTTP request failed: {0}")]
  Request(#[from] reqwest::Error),

  #[error("Tailscale API error ({status}): {message}")]
  Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct TailscaleClient {
  client: Client,
  base_url: String,
  api_key: String,
  tailnet: String,
}

impl TailscaleClient {
  pub fn new(api_key: impl Into<String>, tailnet: impl Into<String>) -> Self {
    Self {
      client: Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build reqwest client"),
      base_url: "https://api.tailscale.com".to_string(),
      api_key: api_key.into(),
      tailnet: tailnet.into(),
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

  pub async fn create_auth_key(&self, request: &CreateKeyRequest) -> Result<Key> {
    let response = self
      .client
      .post(format!(
        "{}/api/v2/tailnet/{}/keys",
        self.base_url, self.tailnet
      ))
      .basic_auth(&self.api_key, Some(""))
      .json(request)
      .send()
      .await?;

    let response = self.check_response(response).await?;
    Ok(response.json().await?)
  }

  pub async fn get_acl(&self) -> Result<Value> {
    let response = self
      .client
      .get(format!(
        "{}/api/v2/tailnet/{}/acl",
        self.base_url, self.tailnet
      ))
      .basic_auth(&self.api_key, Some(""))
      .header("Accept", "application/json")
      .send()
      .await?;

    let response = self.check_response(response).await?;
    Ok(response.json().await?)
  }

  pub async fn list_devices(&self) -> Result<Vec<Device>> {
    let response = self
      .client
      .get(format!(
        "{}/api/v2/tailnet/{}/devices",
        self.base_url, self.tailnet
      ))
      .basic_auth(&self.api_key, Some(""))
      .send()
      .await?;

    let response = self.check_response(response).await?;
    let body: DevicesResponse = response.json().await?;
    Ok(body.devices)
  }

  pub async fn delete_device(&self, device_id: &str) -> Result<()> {
    let response = self
      .client
      .delete(format!("{}/api/v2/device/{}", self.base_url, device_id))
      .basic_auth(&self.api_key, Some(""))
      .send()
      .await?;

    self.check_response(response).await?;
    Ok(())
  }

  pub async fn find_device_by_hostname(&self, hostname: &str) -> Result<Option<Device>> {
    let devices = self.list_devices().await?;
    Ok(devices.into_iter().find(|d| d.hostname == hostname))
  }
}

#[derive(Debug, Serialize)]
pub struct CreateKeyRequest {
  pub capabilities: KeyCapabilities,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expiry_seconds: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct KeyCapabilities {
  pub devices: DeviceCapabilities,
}

#[derive(Debug, Serialize)]
pub struct DeviceCapabilities {
  pub create: DeviceCreateCapabilities,
}

#[derive(Debug, Serialize)]
pub struct DeviceCreateCapabilities {
  pub reusable: bool,
  pub ephemeral: bool,
  pub preauthorized: bool,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Key {
  pub id: String,
  #[serde(default)]
  pub key: String,
  #[serde(default)]
  pub created: Option<String>,
  #[serde(default)]
  pub expires: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DevicesResponse {
  devices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
  pub id: String,
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub hostname: String,
  #[serde(default)]
  pub os: String,
  #[serde(default)]
  pub addresses: Vec<String>,
  #[serde(default)]
  pub authorized: bool,
  #[serde(rename = "lastSeen", default)]
  pub last_seen: Option<String>,
  #[serde(default)]
  pub tags: Vec<String>,
}
