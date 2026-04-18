use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

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
  oauth: OAuthClient,
  tailnet: String,
}

#[derive(Clone)]
struct OAuthClient {
  client_id: String,
  client_secret: String,
  token_cache: Arc<Mutex<Option<CachedToken>>>,
}

#[derive(Clone)]
struct CachedToken {
  access_token: String,
  expires_at: Instant,
}

impl TailscaleClient {
  pub fn new(
    client_id: impl Into<String>,
    client_secret: impl Into<String>,
    tailnet: impl Into<String>,
  ) -> Self {
    Self {
      client: Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build reqwest client"),
      base_url: "https://api.tailscale.com".to_string(),
      oauth: OAuthClient {
        client_id: client_id.into(),
        client_secret: client_secret.into(),
        token_cache: Arc::new(Mutex::new(None)),
      },
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

  async fn send_authenticated(
    &self,
    request: reqwest::RequestBuilder,
  ) -> Result<reqwest::Response> {
    let access_token = self
      .oauth
      .access_token(&self.client, &self.base_url)
      .await?;
    let response = request.bearer_auth(access_token).send().await?;

    self.check_response(response).await
  }

  pub async fn create_auth_key(&self, request: &CreateKeyRequest) -> Result<Key> {
    let response = self
      .send_authenticated(
        self
          .client
          .post(format!(
            "{}/api/v2/tailnet/{}/keys",
            self.base_url, self.tailnet
          ))
          .json(request),
      )
      .await?;

    Ok(response.json().await?)
  }

  pub async fn get_acl(&self) -> Result<Value> {
    let response = self
      .send_authenticated(
        self
          .client
          .get(format!(
            "{}/api/v2/tailnet/{}/acl",
            self.base_url, self.tailnet
          ))
          .header("Accept", "application/json"),
      )
      .await?;

    Ok(response.json().await?)
  }

  pub async fn list_devices(&self) -> Result<Vec<Device>> {
    let response = self
      .send_authenticated(self.client.get(format!(
        "{}/api/v2/tailnet/{}/devices",
        self.base_url, self.tailnet
      )))
      .await?;

    let body: DevicesResponse = response.json().await?;
    Ok(body.devices)
  }

  pub async fn delete_device(&self, device_id: &str) -> Result<()> {
    self
      .send_authenticated(
        self
          .client
          .delete(format!("{}/api/v2/device/{}", self.base_url, device_id)),
      )
      .await?;

    Ok(())
  }

  pub async fn find_device_by_hostname(&self, hostname: &str) -> Result<Option<Device>> {
    let devices = self.list_devices().await?;
    Ok(devices.into_iter().find(|d| d.hostname == hostname))
  }
}

impl OAuthClient {
  async fn access_token(&self, client: &Client, base_url: &str) -> Result<String> {
    let mut token_cache = self.token_cache.lock().await;
    if let Some(token) = token_cache.as_ref() {
      if token.expires_at > Instant::now() + Duration::from_secs(60) {
        return Ok(token.access_token.clone());
      }
    }

    let token_response: OAuthTokenResponse = client
      .post(format!("{}/api/v2/oauth/token", base_url))
      .form(&OAuthTokenRequest {
        client_id: &self.client_id,
        client_secret: &self.client_secret,
        grant_type: "client_credentials",
      })
      .send()
      .await?
      .error_for_status()?
      .json()
      .await?;

    let access_token = token_response.access_token;
    *token_cache = Some(CachedToken {
      access_token: access_token.clone(),
      expires_at: Instant::now() + Duration::from_secs(token_response.expires_in),
    });

    Ok(access_token)
  }
}

#[derive(Serialize)]
struct OAuthTokenRequest<'a> {
  client_id: &'a str,
  client_secret: &'a str,
  grant_type: &'a str,
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
  access_token: String,
  expires_in: u64,
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
