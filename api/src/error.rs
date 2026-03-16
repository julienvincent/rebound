use aide::OperationIo;
use axum::http::StatusCode;
use axum::{Json, response::IntoResponse};
use schemars::JsonSchema;

#[derive(Debug, JsonSchema, serde::Serialize)]
pub struct Fault {
  pub message: String,
}

#[derive(Debug, JsonSchema, serde::Serialize, OperationIo)]
#[aide(output)]
pub enum ApiError {
  #[aide(status = 500)]
  Fault(Fault),
  #[aide(status = 400)]
  BadRequest(Fault),
  #[aide(status = 404)]
  NotFound(Fault),
}

impl IntoResponse for ApiError {
  fn into_response(self) -> axum::response::Response {
    match self {
      Self::Fault(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response(),
      Self::BadRequest(err) => (StatusCode::BAD_REQUEST, Json(err)).into_response(),
      Self::NotFound(err) => (StatusCode::NOT_FOUND, Json(err)).into_response(),
    }
  }
}

impl From<anyhow::Error> for ApiError {
  fn from(value: anyhow::Error) -> Self {
    log::error!("{value:?}");
    ApiError::Fault(Fault {
      message: value.to_string(),
    })
  }
}

impl ApiError {
  pub fn bad_request(message: impl Into<String>) -> Self {
    ApiError::BadRequest(Fault {
      message: message.into(),
    })
  }

  pub fn not_found(message: impl Into<String>) -> Self {
    ApiError::NotFound(Fault {
      message: message.into(),
    })
  }
}
