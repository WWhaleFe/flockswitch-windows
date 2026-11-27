//! Key rule edits left by the settings process for the network process.
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub type Rules = BTreeMap<String, flock_net::DirRules>;

/// A master publishes its complete effective table, including unspecified defaults.
pub fn echoed(keys: &Rules, master: &Rules) -> bool {
    keys.iter()
        .all(|(pair, rule)| master.get(pair) == Some(rule))
}

/// Whether pending edits can never be delivered and must go: this is not a sub with a
/// master any more, or the connected master is a 2.0 one that ignores key rule edits.
pub fn undeliverable(config: &flock_net::NetConfig, status: Option<&flock_net::Status>) -> bool {
    let Some(master) = config.peers.iter().find(|p| p.role == flock_net::Role::Master) else { return true };
    config.role != flock_net::Role::Sub
        || status.is_some_and(|s| s.peers.iter().any(|p| p.id == master.id && p.connected && !p.keyrules))
}

fn read(path: &Path) -> anyhow::Result<Option<Rules>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Settings and network processes serialize access through the same file lock.
/// Serialize consumer calls when the service hook and network thread share this outbox.
pub struct Outbox {
    path: PathBuf,
}

impl Outbox {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn lock(&self) -> anyhow::Result<std::fs::File> {
        if let Some(parent) = self.path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        // Keep this file in place so every process locks the same file. The OS
        // releases the lock on close or process exit, including after a crash.
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.with_extension("lock"))?;
        let started = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(std::fs::TryLockError::WouldBlock) => {
                    if started.elapsed() >= Duration::from_secs(2) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "timed out locking key rule outbox",
                        ).into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(std::fs::TryLockError::Error(e)) => return Err(e.into()),
            }
        }
    }

    /// Queues only the pairs the user changed. Successive edits merge per pair, the newest
    /// value of a pair wins.
    pub fn publish(&self, changed: &Rules) -> anyhow::Result<()> {
        self.publish_with(changed, || {})
    }

    fn publish_with(&self, changed: &Rules, after_read: impl FnOnce()) -> anyhow::Result<()> {
        use std::sync::atomic::{AtomicU64, Ordering};
        let _lock = self.lock()?;
        let mut keys = read(&self.path).ok().flatten().unwrap_or_default();
        after_read();
        keys.extend(changed.iter().map(|(pair, rule)| (pair.clone(), rule.clone())));
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let tmp = self.path.with_extension(format!(
            "{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let result = (|| {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp)?;
            file.write_all(&serde_json::to_vec(&keys)?)?;
            file.sync_all()?;
            drop(file);
            std::fs::rename(&tmp, &self.path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        result
    }

    /// Finish the in-flight edit before claiming the latest queued edit. Serializing
    /// sends prevents a stale snapshot from acknowledging a fast edit-and-revert.
    pub fn pending(&self) -> anyhow::Result<Option<Rules>> {
        let _lock = self.lock()?;
        let inflight = self.path.with_extension("inflight");
        if let Some(keys) = read(&inflight)? {
            return Ok(Some(keys));
        }
        match std::fs::rename(&self.path, &inflight) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        read(&inflight)
    }

    /// Drops the in-flight and the queued edit: unpair, role change, reset, or a master
    /// that cannot take edits.
    pub fn clear(&self) {
        let _lock = match self.lock() {
            Ok(lock) => lock,
            Err(e) => {
                log::warn!("lock pending key rules for clearing: {e:#}");
                return;
            }
        };
        for path in [self.path.clone(), self.path.with_extension("inflight")] {
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => log::warn!("clear pending key rules: {e}"),
            }
        }
    }

    /// The settings UI keeps showing its newest edit while an older send completes.
    pub fn latest(&self) -> anyhow::Result<Option<Rules>> {
        let _lock = self.lock()?;
        let sent = read(&self.path.with_extension("inflight"))?;
        let queued = read(&self.path)?;
        Ok(match (sent, queued) {
            (Some(mut sent), Some(queued)) => { sent.extend(queued); Some(sent) }
            (sent, queued) => sent.or(queued),
        })
    }

    /// Only rules observed in the master's settings count as confirmation, never a
    /// successful enqueue of the outbound service command.
    pub fn confirm(&self, keys: &Rules) -> anyhow::Result<()> {
        let _lock = self.lock()?;
        let inflight = self.path.with_extension("inflight");
        if read(&inflight)?
            .as_ref()
            .is_some_and(|pending| echoed(pending, keys))
        {
            std::fs::remove_file(inflight)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let dir = std::env::current_dir()
                .unwrap()
                .join("target")
                .join(format!(
                    "flockswitch-key-rules-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn outbox(&self) -> Outbox {
            Outbox::new(self.0.join("key-rules.request"))
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn rules(mode: flock_net::ModMode) -> Rules {
        let mut config = flock_net::NetConfig::default();
        config.migrate_keys();
        config.keys.get_mut("mac>windows").unwrap().mod_mode = mode;
        config.keys
    }

    #[test]
    fn publishing_cannot_resurrect_an_edit_claimed_and_confirmed_during_the_merge() {
        use std::{sync::mpsc, thread, time::Duration};

        let fixture = Fixture::new();
        let mut old = rules(flock_net::ModMode::Off);
        old.retain(|pair, _| pair == "mac>windows");
        let mut new = rules(flock_net::ModMode::Position);
        new.retain(|pair, _| pair == "linux>windows");
        fixture.outbox().publish(&old).unwrap();
        let (read_tx, read_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let publisher = fixture.outbox();
        let consumer = fixture.outbox();
        let (completed_during_merge, delivered) = thread::scope(|scope| {
            let network = scope.spawn(move || {
                read_rx.recv().unwrap();
                let delivered = consumer.pending().unwrap().unwrap();
                consumer.confirm(&delivered).unwrap();
                done_tx.send(()).unwrap();
                delivered
            });
            let mut completed_during_merge = false;
            publisher.publish_with(&new, || {
                read_tx.send(()).unwrap();
                completed_during_merge = done_rx.recv_timeout(Duration::from_millis(250)).is_ok();
            }).unwrap();
            // Keep the receiver alive until the consumer has finished after publication.
            let delivered = network.join().unwrap();
            (completed_during_merge, delivered)
        });
        assert_eq!(fixture.outbox().pending().unwrap(), None,
            "an acknowledged old edit must not be queued again by the publisher");
        old.extend(new);
        assert_eq!(delivered, old);
        assert!(!completed_during_merge);
    }

    #[test]
    fn every_outbox_operation_waits_for_the_same_file_lock() {
        use std::{sync::mpsc, thread};

        for operation in ["publish", "pending", "confirm", "clear", "latest"] {
            let fixture = Fixture::new();
            let keys = rules(flock_net::ModMode::Off);
            fixture.outbox().publish(&keys).unwrap();
            fixture.outbox().pending().unwrap();
            let lock = fixture.outbox().lock().unwrap();
            let worker = fixture.outbox();
            let (started_tx, started_rx) = mpsc::channel();
            let (done_tx, done_rx) = mpsc::channel();
            thread::scope(|scope| {
                let task = scope.spawn(move || {
                    started_tx.send(()).unwrap();
                    match operation {
                        "publish" => worker.publish(&keys).unwrap(),
                        "pending" => { worker.pending().unwrap(); }
                        "confirm" => worker.confirm(&keys).unwrap(),
                        "clear" => worker.clear(),
                        "latest" => { worker.latest().unwrap(); }
                        _ => unreachable!(),
                    }
                    done_tx.send(()).unwrap();
                });
                started_rx.recv().unwrap();
                let result = done_rx.recv_timeout(Duration::from_millis(100));
                // Always release before asserting so a failure cannot strand the worker.
                drop(lock);
                task.join().unwrap();
                assert!(matches!(result, Err(mpsc::RecvTimeoutError::Timeout)),
                    "{operation} bypassed the outbox lock: {result:?}");
                done_rx.recv().unwrap();
            });
        }
    }

    #[test]
    fn lock_contention_times_out_without_removing_the_live_lock() {
        let fixture = Fixture::new();
        let lock = fixture.outbox().lock().unwrap();
        let error = fixture.outbox().publish(&rules(flock_net::ModMode::Off)).unwrap_err();
        assert_eq!(error.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::TimedOut);
        let contender = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(fixture.outbox().path.with_extension("lock"))
            .unwrap();
        assert!(matches!(contender.try_lock(), Err(std::fs::TryLockError::WouldBlock)));
        assert!(!fixture.outbox().path.exists());
        drop(lock);
        // An existing lock file is reusable immediately after its owner closes it.
        fixture.outbox().publish(&rules(flock_net::ModMode::Position)).unwrap();
        assert_eq!(fixture.outbox().pending().unwrap(), Some(rules(flock_net::ModMode::Position)));
    }

    #[test]
    fn offline_edits_survive_retries_and_process_restart_until_master_echoes_them() {
        let fixture = Fixture::new();
        let keys = rules(flock_net::ModMode::Off);
        fixture.outbox().publish(&keys).unwrap();
        assert_eq!(fixture.outbox().pending().unwrap(), Some(keys.clone()));
        // No service, no link, or a successful enqueue without a reply must all retry.
        assert_eq!(fixture.outbox().pending().unwrap(), Some(keys.clone()));
        fixture
            .outbox()
            .confirm(&rules(flock_net::ModMode::Position))
            .unwrap();
        assert_eq!(fixture.outbox().pending().unwrap(), Some(keys.clone()));
        fixture.outbox().confirm(&keys).unwrap();
        assert_eq!(fixture.outbox().pending().unwrap(), None);
    }

    #[test]
    fn clear_drops_queued_and_in_flight_edits() {
        let fixture = Fixture::new();
        let outbox = fixture.outbox();
        outbox.publish(&rules(flock_net::ModMode::Off)).unwrap();
        outbox.pending().unwrap();
        outbox.publish(&rules(flock_net::ModMode::Position)).unwrap();
        outbox.clear();
        outbox.clear();
        assert_eq!(outbox.pending().unwrap(), None);
        assert_eq!(outbox.latest().unwrap(), None);
    }

    #[test]
    fn edits_are_undeliverable_without_a_sub_role_a_master_or_a_keyrules_master() {
        use flock_net::{MeStatus, NetConfig, PeerConfig, PeerStatus, Role, StateStatus, Status};
        let mut config = NetConfig::default();
        config.role = Role::Sub;
        assert!(undeliverable(&config, None));
        config.peers.push(PeerConfig { id: "m".into(), role: Role::Master, ..Default::default() });
        assert!(!undeliverable(&config, None));
        let status = |connected, keyrules| Status {
            running: true,
            me: MeStatus { id: String::new(), name: String::new(), port: 0, error: None, can_send: true },
            discovered: vec![],
            peers: vec![PeerStatus { id: "m".into(), name: "M".into(), os: None, connected, rtt_ms: None, can_send: true, via_master: false, keyrules }],
            state: StateStatus { mode: "local".into(), peer: None },
            pairing: None, transfer: None, following: None, layout: vec![], reconnecting: false, keys: Default::default(),
        };
        assert!(!undeliverable(&config, Some(&status(true, true))));
        assert!(!undeliverable(&config, Some(&status(false, false))));
        assert!(undeliverable(&config, Some(&status(true, false))));
        config.role = Role::Master;
        assert!(undeliverable(&config, None));
    }

    #[test]
    fn an_old_reply_cannot_delete_an_edit_published_after_the_read() {
        let fixture = Fixture::new();
        let old = rules(flock_net::ModMode::Off);
        let new = rules(flock_net::ModMode::Position);
        let outbox = fixture.outbox();
        outbox.publish(&old).unwrap();
        assert_eq!(outbox.pending().unwrap(), Some(old.clone()));
        outbox.publish(&new).unwrap();
        outbox.confirm(&old).unwrap();
        assert_eq!(outbox.pending().unwrap(), Some(new.clone()));
        outbox.confirm(&old).unwrap();
        assert_eq!(outbox.pending().unwrap(), Some(new.clone()));
        outbox.confirm(&new).unwrap();
        assert_eq!(outbox.pending().unwrap(), None);
    }

    #[test]
    fn a_revert_waits_for_the_previous_edit_to_be_confirmed() {
        let fixture = Fixture::new();
        let outbox = fixture.outbox();
        let first = rules(flock_net::ModMode::Off);
        outbox.publish(&first).unwrap();
        assert_eq!(outbox.pending().unwrap(), Some(first.clone()));
        let latest = rules(flock_net::ModMode::Position);
        outbox.publish(&latest).unwrap();
        // The master still reports Position while Off is in transit. That old
        // snapshot must not acknowledge the queued revert to Position.
        outbox.confirm(&latest).unwrap();
        assert_eq!(outbox.pending().unwrap(), Some(first.clone()));
        assert_eq!(outbox.latest().unwrap(), Some(latest.clone()));
        outbox.confirm(&first).unwrap();
        assert_eq!(outbox.pending().unwrap(), Some(latest.clone()));
        // A retry of the old acknowledgement cannot consume the revert either.
        outbox.confirm(&first).unwrap();
        assert_eq!(outbox.pending().unwrap(), Some(latest.clone()));
        outbox.confirm(&latest).unwrap();
        assert_eq!(outbox.pending().unwrap(), None);
    }

    #[test]
    fn successive_edits_merge_per_pair_and_latest_covers_the_one_in_flight() {
        let fixture = Fixture::new();
        let outbox = fixture.outbox();
        let one = |pair: &str, mode| -> Rules {
            let mut keys = rules(mode);
            keys.retain(|key, _| key == pair);
            keys
        };
        outbox.publish(&one("mac>windows", flock_net::ModMode::Off)).unwrap();
        outbox.publish(&one("linux>windows", flock_net::ModMode::Off)).unwrap();
        outbox.publish(&one("mac>windows", flock_net::ModMode::Position)).unwrap();
        let pending = outbox.pending().unwrap().unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending["mac>windows"].mod_mode, flock_net::ModMode::Position);
        outbox.publish(&one("mac>android", flock_net::ModMode::Off)).unwrap();
        let latest = outbox.latest().unwrap().unwrap();
        assert_eq!(latest.len(), 3);
        assert_eq!(outbox.pending().unwrap().unwrap().len(), 2);
    }

    #[test]
    fn partial_requests_are_confirmed_by_the_masters_complete_rule_table() {
        let fixture = Fixture::new();
        let outbox = fixture.outbox();
        let full = rules(flock_net::ModMode::Off);
        let partial = full
            .iter()
            .filter(|(key, _)| key.as_str() == "mac>windows")
            .map(|(key, rule)| (key.clone(), rule.clone()))
            .collect();
        outbox.publish(&partial).unwrap();
        outbox.pending().unwrap();
        outbox.confirm(&full).unwrap();
        assert_eq!(outbox.pending().unwrap(), None);
    }
}
