use std::fmt;

#[derive(Debug)]
pub struct ApiError {
  pub status: u16,
  pub message: String,
}

impl ApiError {
  pub fn from_response(status: u16, body: &str) -> Self {
    let message = serde_json::from_str::<serde_json::Value>(body)
      .ok()
      .and_then(|v| v.get("message")?.as_str().map(|s| s.to_string()))
      .unwrap_or_else(|| body.to_string());

    Self { status, message }
  }
}

impl fmt::Display for ApiError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "API error ({}): {}", self.status, self.message)
  }
}

impl std::error::Error for ApiError {}
