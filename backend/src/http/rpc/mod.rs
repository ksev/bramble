use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use tokio::sync::mpsc::Sender;

use super::pipe::PipeRequest;

mod protocol;
mod zigbee2mqtt;

pub static ROUTER: Lazy<Router> = Lazy::new(Router::default);

/// Trait every RPC service has to implement to be routable
#[async_trait::async_trait]
pub trait ServiceRouter {
    fn id(&self) -> u16;
    async fn route(&self, req: PipeRequest, tx: Sender<Vec<u8>>) -> anyhow::Result<()>;
}

/// RPC router, routes requests from the clients to the correct service and call
pub struct Router {
    service_routes: BTreeMap<u16, Box<dyn ServiceRouter + Sync + Send>>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Router {
        Router {
            service_routes: BTreeMap::new(),
        }
    }

    /// Add a RPC service to the router
    pub fn add_service(&mut self, service: Box<dyn ServiceRouter + Sync + Send>) {
        let id = service.id();
        self.service_routes.insert(id, service);
    }

    /// Route between the different services
    pub async fn route(&self, req: PipeRequest, response: Sender<Vec<u8>>) -> anyhow::Result<()> {
        if let Some(s) = self.service_routes.get(&req.service_id) {
            Ok(s.route(req, response).await?)
        } else {
            anyhow::bail!("invalid service_id");
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        let mut router = Router::new();
        router.add_service(Box::new(protocol::Zigbee2MqttServiceRouter::<
            zigbee2mqtt::Service,
        >::default()));
        router
    }
}
