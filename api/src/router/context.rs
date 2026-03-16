use std::sync::Arc;

use crate::system::SystemApi;
use fly_client::FlyClient;
use tailscale_client::TailscaleClient;

#[derive(Clone)]
pub struct Context {
  system: Arc<Box<dyn SystemApi + Send + Sync + 'static>>,
}

impl Context {
  pub fn new(system: impl SystemApi + Send + Sync + 'static) -> Self {
    Self {
      system: Arc::new(Box::new(system)),
    }
  }
}

impl SystemApi for Context {
  fn pool(&self) -> &deadpool_postgres::Pool {
    self.system.pool()
  }

  fn fly(&self) -> &FlyClient {
    self.system.fly()
  }

  fn tailscale(&self) -> &TailscaleClient {
    self.system.tailscale()
  }

  fn exit_node_tag(&self) -> &str {
    self.system.exit_node_tag()
  }
}
