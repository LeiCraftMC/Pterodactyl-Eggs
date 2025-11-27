use std::sync::Arc;

use async_trait::async_trait;
use pingora::prelude::*;

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:19130";
const DEFAULT_WORLD_BACKENDS: [&str; 2] = ["127.0.0.1:19131", "127.0.0.1:19132"];
const SUPERVISOR_BACKEND: &str = "127.0.0.1:19180";

#[derive(Clone)]
pub struct SupervisorProxy {
    world_lb: Arc<LoadBalancer<RoundRobin>>,
    supervisor_backend: HttpPeer,
}

impl SupervisorProxy {
    pub fn new(world_lb: Arc<LoadBalancer<RoundRobin>>, supervisor_backend: HttpPeer) -> Self {
        Self {
            world_lb,
            supervisor_backend,
        }
    }

    fn select_world_backend(&self, path: &str) -> Result<Box<HttpPeer>> {
        let hash_input = path.as_bytes();
        let backend = self
            .world_lb
            .select(hash_input, 256)
            .ok_or_else(|| Error::new(ErrorType::InternalError))?;

        Ok(Box::new(HttpPeer::new(backend, false, String::new())))
    }
}

#[async_trait]
impl ProxyHttp for SupervisorProxy {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();

        if path.starts_with("/_supervisor") {
            // Route supervisor control traffic to the management API endpoint.
            return Ok(Box::new(self.supervisor_backend.clone()));
        }

        self.select_world_backend(path)
    }
}

pub fn start_proxy() -> Result<()> {
    let listen_addr = std::env::var("SUPERVISOR_PROXY_LISTEN")
        .unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string());

    let world_backends = Arc::new(
        LoadBalancer::try_from_iter(DEFAULT_WORLD_BACKENDS)
            .expect("world backend addresses must be valid socket addresses"),
    );

    let supervisor_backend = HttpPeer::new(SUPERVISOR_BACKEND, false, String::new());
    let app = SupervisorProxy::new(world_backends, supervisor_backend);

    let mut server = Server::new(None)?;
    server.bootstrap();

    let mut proxy_service = http_proxy_service(&server.configuration, app);
    proxy_service.add_tcp(&listen_addr);
    tracing::info!(target: "supervisor", "pingora reverse proxy listening on {listen_addr}");

    server.add_service(proxy_service);
    server.run_forever()
}
