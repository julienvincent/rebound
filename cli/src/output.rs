use anyhow::Result;
use serde::Serialize;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::OutputFormat;

pub fn render_list<T: Tabled + Serialize>(
  rows: Vec<T>,
  output: &OutputFormat,
  empty_msg: &str,
) -> Result<()> {
  match output {
    OutputFormat::Json => {
      println!("{}", serde_json::to_string_pretty(&rows)?);
    }
    OutputFormat::Table => {
      if rows.is_empty() {
        println!("{empty_msg}");
      } else {
        println!("{}", Table::new(rows).with(Style::rounded()));
      }
    }
  }
  Ok(())
}

pub fn render_one<T: Tabled + Serialize>(item: T, output: &OutputFormat) -> Result<()> {
  match output {
    OutputFormat::Json => {
      println!("{}", serde_json::to_string_pretty(&item)?);
    }
    OutputFormat::Table => {
      println!("{}", Table::new([item]).with(Style::rounded()));
    }
  }
  Ok(())
}

pub fn render_json<T: Serialize>(item: &T, output: &OutputFormat) -> Result<()> {
  match output {
    OutputFormat::Json => {
      println!("{}", serde_json::to_string_pretty(item)?);
    }
    OutputFormat::Table => {
      println!("{}", serde_json::to_string_pretty(item)?);
    }
  }
  Ok(())
}
