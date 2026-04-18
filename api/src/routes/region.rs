use axum::Json;
use schemars::JsonSchema;

use crate::error::ApiError;

#[derive(Debug, serde::Serialize, JsonSchema)]
pub struct Region {
  pub code: &'static str,
  pub name: &'static str,
}

#[derive(Debug, serde::Serialize, JsonSchema, aide::OperationIo)]
pub struct ListRegionsResponse {
  pub regions: Vec<Region>,
}

pub static REGIONS: &[Region] = &[
  Region {
    code: "ams",
    name: "Amsterdam, Netherlands",
  },
  Region {
    code: "arn",
    name: "Stockholm, Sweden",
  },
  Region {
    code: "bom",
    name: "Mumbai, India",
  },
  Region {
    code: "cdg",
    name: "Paris, France",
  },
  Region {
    code: "dfw",
    name: "Dallas, Texas (US)",
  },
  Region {
    code: "ewr",
    name: "Secaucus, NJ (US)",
  },
  Region {
    code: "fra",
    name: "Frankfurt, Germany",
  },
  Region {
    code: "gru",
    name: "São Paulo, Brazil",
  },
  Region {
    code: "iad",
    name: "Ashburn, Virginia (US)",
  },
  Region {
    code: "jnb",
    name: "Johannesburg, South Africa",
  },
  Region {
    code: "lax",
    name: "Los Angeles, California (US)",
  },
  Region {
    code: "lhr",
    name: "London, United Kingdom",
  },
  Region {
    code: "nrt",
    name: "Tokyo, Japan",
  },
  Region {
    code: "ord",
    name: "Chicago, Illinois (US)",
  },
  Region {
    code: "sin",
    name: "Singapore, Singapore",
  },
  Region {
    code: "sjc",
    name: "San Jose, California (US)",
  },
  Region {
    code: "syd",
    name: "Sydney, Australia",
  },
  Region {
    code: "yyz",
    name: "Toronto, Canada",
  },
];

pub async fn list_regions() -> Result<Json<ListRegionsResponse>, ApiError> {
  let regions = REGIONS
    .iter()
    .map(|r| Region {
      code: r.code,
      name: r.name,
    })
    .collect();

  Ok(Json(ListRegionsResponse { regions }))
}
