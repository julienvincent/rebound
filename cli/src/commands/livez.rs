use anyhow::Result;

use crate::client::Client;

pub fn run(client: &Client, rt: &tokio::runtime::Runtime) -> Result<()> {
  let text = rt.block_on(client.get_text("/livez"))?;
  println!("{text}");
  Ok(())
}
