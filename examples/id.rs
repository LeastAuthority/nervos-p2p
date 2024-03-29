use env_logger;
use log::debug;

use futures::{future::lazy, prelude::*};
use identify::{Callback, IdentifyProtocol, MisbehaveResult, Misbehavior};
use tentacle::{
    builder::{MetaBuilder, ServiceBuilder},
    context::{ProtocolContextMutRef, ServiceContext},
    multiaddr::Multiaddr,
    secio::{PeerId, SecioKeyPair},
    service::{DialProtocol, ProtocolHandle, ServiceError, ServiceEvent, SessionType},
    traits::ServiceHandle,
};

fn main() {
    env_logger::init();
    let callback = IdentifyCallback {
        local_listen_addrs: Vec::new(),
    };
    let protocol = MetaBuilder::default()
        .id(1.into())
        .service_handle(move || ProtocolHandle::Callback(Box::new(IdentifyProtocol::new(callback))))
        .build();
    if std::env::args().nth(1) == Some("server".to_string()) {
        debug!("Starting server ......");
        let mut service = ServiceBuilder::default()
            .insert_protocol(protocol)
            .key_pair(SecioKeyPair::secp256k1_generated())
            .forever(true)
            .build(SimpleHandler {});
        let _ = service.dial(
            "/ip4/127.0.0.1/tcp/1338".parse().unwrap(),
            DialProtocol::All,
        );
        let _ = service.listen("/ip4/127.0.0.1/tcp/1337".parse().unwrap());
        tokio::run(lazy(|| service.for_each(|_| Ok(()))))
    } else {
        debug!("Starting client ......");
        let mut service = ServiceBuilder::default()
            .insert_protocol(protocol)
            .key_pair(SecioKeyPair::secp256k1_generated())
            .forever(true)
            .build(SimpleHandler {});
        let _ = service.dial(
            "/ip4/127.0.0.1/tcp/1337".parse().unwrap(),
            DialProtocol::All,
        );
        let _ = service.listen("/ip4/127.0.0.1/tcp/1338".parse().unwrap());
        tokio::run(lazy(|| service.for_each(|_| Ok(()))))
    }
}

#[derive(Clone)]
struct IdentifyCallback {
    local_listen_addrs: Vec<Multiaddr>,
}

impl Callback for IdentifyCallback {
    fn identify(&mut self) -> &[u8] {
        "Identify message".as_bytes()
    }

    fn received_identify(
        &mut self,
        _context: &mut ProtocolContextMutRef,
        identify: &[u8],
    ) -> MisbehaveResult {
        log::info!("{}", String::from_utf8_lossy(identify));
        MisbehaveResult::Continue
    }

    /// Get local listen addresses
    fn local_listen_addrs(&mut self) -> Vec<Multiaddr> {
        self.local_listen_addrs.clone()
    }
    /// Add remote peer's listen addresses
    fn add_remote_listen_addrs(&mut self, _peer: &PeerId, _addrs: Vec<Multiaddr>) {}
    /// Add our address observed by remote peer
    fn add_observed_addr(
        &mut self,
        _peer: &PeerId,
        _addr: Multiaddr,
        _: SessionType,
    ) -> MisbehaveResult {
        MisbehaveResult::Continue
    }
    /// Report misbehavior
    fn misbehave(&mut self, _peer: &PeerId, _kind: Misbehavior) -> MisbehaveResult {
        MisbehaveResult::Disconnect
    }
}

struct SimpleHandler {}

impl ServiceHandle for SimpleHandler {
    fn handle_error(&mut self, _context: &mut ServiceContext, error: ServiceError) {
        debug!("service error: {:?}", error);
    }

    fn handle_event(&mut self, _context: &mut ServiceContext, event: ServiceEvent) {
        debug!("service event: {:?}", event);
    }
}
