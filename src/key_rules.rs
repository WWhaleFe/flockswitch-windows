//! Key rule edits left by the settings process for the network process.
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};

pub type Rules = BTreeMap<String, flock_net::DirRules>;

/// A master publishes its complete effective table, including unspecified defaults.
pub fn echoed(keys: &Rules, master: &Rules) -> bool {
    keys.iter()
        .all(|(pair, rule)| master.get(pair) == Some(rule))
}

fn read(path: &Path) -> anyhow::Result<Option<Rules>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// One producer publishes edits; one consumer owns the in-flight file. Serialize
/// consumer calls when the service hook and network thread share this outbox.
pub struct Outbox {
    path: PathBuf,
}

impl Outbox {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn publish(&self, keys: &Rules) -> anyhow::Result<()> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
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
            file.write_all(&serde_json::to_vec(keys)?)?;
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

    /// The settings UI keeps showing its newest edit while an older send completes.
    pub fn latest(&self) -> anyhow::Result<Option<Rules>> {
        match read(&self.path)? {
            Some(keys) => Ok(Some(keys)),
            None => read(&self.path.with_extension("inflight")),
        }
    }

    /// Only rules observed in the master's settings count as confirmation, never a
    /// successful enqueue of the outbound service command.
    pub fn confirm(&self, keys: &Rules) -> anyhow::Result<()> {
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
