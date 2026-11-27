//! Names of other computers for the tray, status card and layout map. Host-testable.
use flock_net::{NetConfig, Status};

/// The name to show for the computer `id`: the live status (also for a sub known only
/// through the master), else the configured peer name. None when neither has one.
pub fn peer_name(net: &NetConfig, status: Option<&Status>, id: &str) -> Option<String> {
    status
        .and_then(|s| s.peers.iter().find(|p| p.id == id))
        .map(|p| p.name.clone())
        .filter(|n| !n.is_empty())
        .or_else(|| net.peers.iter().find(|p| p.id == id).map(|p| p.name.clone()).filter(|n| !n.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flock_net::{MeStatus, PeerConfig, PeerStatus, StateStatus};

    fn peer(id: &str, name: &str) -> PeerStatus {
        PeerStatus { id: id.into(), name: name.into(), os: None, connected: true, rtt_ms: None, can_send: true, via_master: true, keyrules: true, displays: vec![], paused: false }
    }

    #[test]
    fn names_prefer_the_live_status_then_the_config_then_nothing() {
        let mut net = NetConfig::default();
        net.peers.push(PeerConfig { id: "p1".into(), name: "Laptop".into(), ..Default::default() });
        let status = Status {
            running: true,
            me: MeStatus { id: String::new(), name: String::new(), port: 0, error: None, can_send: true, displays: vec![] },
            discovered: vec![],
            peers: vec![peer("p1", "Renamed"), peer("p2", "Desk"), peer("p3", "")],
            state: StateStatus { mode: "local".into(), peer: None },
            pairing: None,
            transfer: None,
            following: None,
            layout: vec![],
            reconnecting: false,
            keys: Default::default(),
        };
        assert_eq!(peer_name(&net, Some(&status), "p1").as_deref(), Some("Renamed"));
        assert_eq!(peer_name(&net, Some(&status), "p2").as_deref(), Some("Desk"));
        assert_eq!(peer_name(&net, Some(&status), "p3"), None);
        assert_eq!(peer_name(&net, None, "p1").as_deref(), Some("Laptop"));
    }
}
