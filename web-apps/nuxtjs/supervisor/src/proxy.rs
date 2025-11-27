use std::{
    net::ToSocketAddrs,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use pingora::{prelude::*, upstreams::peer::Peer};

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:19130";
const DEFAULT_WORLD_BACKEND: &str = "127.0.0.1:19131";
const SUPERVISOR_BACKEND: &str = "127.0.0.1:19180";

static WORLD_BACKEND: Lazy<Arc<RwLock<HttpPeer>>> = Lazy::new(|| {
    Arc::new(RwLock::new(HttpPeer::new(
        DEFAULT_WORLD_BACKEND,
        false,
        String::new(),
    )))
});

#[derive(Clone)]
pub struct SupervisorProxy {
    world_backend: Arc<RwLock<HttpPeer>>,
    supervisor_backend: HttpPeer,
}

impl SupervisorProxy {
    pub fn new(world_backend: Arc<RwLock<HttpPeer>>, supervisor_backend: HttpPeer) -> Self {
        Self {
            world_backend,
            supervisor_backend,
        }
    }

    fn current_world_peer(&self) -> Result<Box<HttpPeer>> {
        let guard = self
            .world_backend
            .read()
            .map_err(|_| Error::new(ErrorType::InternalError))?;
        Ok(Box::new(guard.clone()))
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

        self.current_world_peer()
    }
}

pub fn start_proxy() -> Result<()> {
    let listen_addr = std::env::var("SUPERVISOR_PROXY_LISTEN")
        .unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string());

    let world_backend = WORLD_BACKEND.clone();
    let supervisor_backend = HttpPeer::new(SUPERVISOR_BACKEND, false, String::new());
    let app = SupervisorProxy::new(world_backend, supervisor_backend);

    let mut server = Server::new(None)?;
    server.bootstrap();

    let mut proxy_service = http_proxy_service(&server.configuration, app);
    proxy_service.add_tcp(&listen_addr);
    tracing::info!(target: "supervisor", "pingora reverse proxy listening on {listen_addr}");

    server.add_service(proxy_service);
    server.run_forever()
}

pub fn set_world_backend(addr: &str) -> Result<()> {
    validate_backend(addr)?;
    let peer = HttpPeer::new(addr, false, String::new());
    let mut guard = WORLD_BACKEND
        .write()
        .map_err(|_| Error::new(ErrorType::InternalError))?;
    *guard = peer;
    tracing::info!(target: "supervisor", "world backend updated to {addr}");
    Ok(())
}

// pub fn get_world_backend() -> Option<String> {
//     WORLD_BACKEND
//         .read()
//         .ok()
//     .map(|peer| peer.address().to_string())
// }

fn validate_backend(addr: &str) -> Result<()> {
    addr.to_socket_addrs()
        .map_err(|_| Error::new(ErrorType::InternalError))?
        .next()
        .ok_or_else(|| Error::new(ErrorType::InternalError))?;
    Ok(())
}
