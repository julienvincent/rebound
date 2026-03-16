use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

use crate::error::ApiError;

pub struct Client {
  http: reqwest::Client,
  base_url: String,
}

impl Client {
  pub fn new(host: &str) -> Self {
    Self {
      http: reqwest::Client::new(),
      base_url: host.trim_end_matches('/').to_string(),
    }
  }

  pub async fn get_text(&self, path: &str) -> Result<String> {
    let url = format!("{}{}", self.base_url, path);
    let response = self.http.get(&url).send().await.context("request failed")?;

    if !response.status().is_success() {
      let status = response.status().as_u16();
      let body = response.text().await.unwrap_or_default();
      return Err(ApiError::from_response(status, &body).into());
    }

    Ok(response.text().await?)
  }

  pub async fn post<B: serde::Serialize, R: DeserializeOwned>(
    &self,
    path: &str,
    body: &B,
  ) -> Result<R> {
    let url = format!("{}{}", self.base_url, path);
    let response = self
      .http
      .post(&url)
      .json(body)
      .send()
      .await
      .context("request failed")?;

    if !response.status().is_success() {
      let status = response.status().as_u16();
      let body = response.text().await.unwrap_or_default();
      return Err(ApiError::from_response(status, &body).into());
    }

    Ok(response.json().await?)
  }
}
