use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use pingora::prelude::{HttpPeer, LoadBalancer, ProxyHttp, RoundRobin, Session, Result};

pub struct LB(pub Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self.0
            .select(b"", 256)
            .unwrap();

        let mut peer = HttpPeer::new(upstream, false, "one.one.one.one".to_string());
        peer.options.idle_timeout = Some(Duration::from_secs(60));

        // Set SNI to one.one.one.one
        let peer = Box::new(peer);
        Ok(peer)
    }
}