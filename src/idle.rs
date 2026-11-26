//! Event waits and short-lived workers for the desktop app.

use std::sync::{
    mpsc::{self, Receiver, Sender},
    Condvar, Mutex,
};
use std::time::Duration;

pub const WORKER_STACK: usize = 512 * 1024;
pub const EDGE_POLL: Duration = Duration::from_millis(25);
// Shakes can turn every 60-150 ms. Keep the original sampling cadence during motion.
pub const SHAKE_POLL: Duration = Duration::from_millis(10);
pub const NETWORK_POLL: Duration = Duration::from_millis(50);
pub const CONFIG_POLL: Duration = Duration::from_millis(250);
pub const WORKER_IDLE: Duration = Duration::from_secs(1);

/// Cursor sampling stays unchanged while needed. Disabled features wait for events.
pub fn cursor_wait(edges: bool, shake: bool) -> Option<Duration> {
    if shake {
        Some(SHAKE_POLL)
    } else if edges {
        Some(EDGE_POLL)
    } else {
        None
    }
}

/// Paused update checks have no timer; configuration and installation changes wake them.
pub fn update_wait(enabled: bool, installing: bool, remaining: Duration) -> Option<Duration> {
    (enabled && !installing).then_some(remaining)
}

/// A generation prevents a notification between checking work and waiting being lost.
#[derive(Default)]
pub struct Signal {
    generation: Mutex<u64>,
    changed: Condvar,
}

impl Signal {
    pub fn revision(&self) -> u64 {
        *self.generation.lock().unwrap()
    }

    pub fn notify(&self) {
        let mut generation = self.generation.lock().unwrap();
        *generation = generation.wrapping_add(1);
        self.changed.notify_all();
    }

    pub fn wait(&self, observed: u64, timeout: Option<Duration>) {
        let generation = self.generation.lock().unwrap();
        if let Some(timeout) = timeout {
            drop(
                self.changed
                    .wait_timeout_while(generation, timeout, |g| *g == observed)
                    .unwrap(),
            );
        } else {
            drop(
                self.changed
                    .wait_while(generation, |g| *g == observed)
                    .unwrap(),
            );
        }
    }
}

/// Sending and retiring share a lock: a command cannot reach a retiring worker.
pub struct WorkerMailbox<T> {
    sender: Mutex<Option<Sender<T>>>,
}

impl<T> Default for WorkerMailbox<T> {
    fn default() -> Self {
        Self {
            sender: Mutex::new(None),
        }
    }
}

impl<T> WorkerMailbox<T> {
    pub const fn new() -> Self {
        Self {
            sender: Mutex::new(None),
        }
    }

    pub fn send_if_running(&self, command: T) {
        let mut sender = self.sender.lock().unwrap();
        if sender
            .as_ref()
            .is_some_and(|active| active.send(command).is_err())
        {
            *sender = None;
        }
    }

    pub fn send(
        &self,
        command: T,
        start: impl FnOnce(Receiver<T>) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        let mut sender = self.sender.lock().unwrap();
        let command = if let Some(active) = sender.as_ref() {
            match active.send(command) {
                Ok(()) => return Ok(()),
                Err(error) => error.0,
            }
        } else {
            command
        };
        *sender = None;
        let (send, receive) = mpsc::channel();
        start(receive)?;
        send.send(command)
            .map_err(|_| std::io::Error::other("worker stopped"))?;
        *sender = Some(send);
        Ok(())
    }

    /// Returns a command that raced with the idle timeout, or releases the sender.
    pub fn take_or_retire(&self, receive: &Receiver<T>) -> Option<T> {
        let mut sender = self.sender.lock().unwrap();
        match receive.try_recv() {
            Ok(command) => Some(command),
            Err(_) => {
                *sender = None;
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn only_active_cursor_features_poll() {
        assert_eq!(cursor_wait(false, false), None);
        assert_eq!(cursor_wait(true, false), Some(Duration::from_millis(25)));
        assert_eq!(cursor_wait(false, true), Some(Duration::from_millis(10)));
        assert_eq!(cursor_wait(true, true), Some(Duration::from_millis(10)));
    }

    #[test]
    fn updates_wait_for_the_deadline_or_a_setting_change() {
        let day = Duration::from_secs(24 * 60 * 60);
        assert_eq!(update_wait(true, false, day), Some(day));
        assert_eq!(
            update_wait(true, false, Duration::ZERO),
            Some(Duration::ZERO)
        );
        assert_eq!(update_wait(false, false, day), None);
        assert_eq!(update_wait(true, true, Duration::ZERO), None);
        assert_eq!(update_wait(false, true, Duration::ZERO), None);
    }

    #[test]
    fn a_notification_before_waiting_is_not_lost() {
        let signal = Signal::default();
        let observed = signal.revision();
        signal.notify();
        signal.wait(observed, None);
        signal.wait(observed, Some(Duration::from_secs(60)));
    }

    #[test]
    fn a_notification_wakes_all_waiters() {
        let signal = Arc::new(Signal::default());
        let observed = signal.revision();
        let (finished, receive) = mpsc::channel();
        let workers: Vec<_> = (0..3)
            .map(|_| {
                let signal = signal.clone();
                let finished = finished.clone();
                std::thread::spawn(move || {
                    signal.wait(observed, None);
                    finished.send(()).unwrap();
                })
            })
            .collect();
        signal.notify();
        for _ in &workers {
            receive.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        for worker in workers {
            worker.join().unwrap();
        }
    }

    #[test]
    fn a_timer_expires_without_an_event() {
        Signal::default().wait(0, Some(Duration::ZERO));
    }

    #[test]
    fn workers_retire_and_restart_without_losing_commands() {
        let mailbox = WorkerMailbox::new();
        mailbox.send_if_running(0);
        let mut receive = None;
        mailbox
            .send(1, |r| {
                receive = Some(r);
                Ok(())
            })
            .unwrap();
        let first = receive.unwrap();
        assert_eq!(first.recv().unwrap(), 1);
        mailbox.send(2, |_| panic!("reuse the worker")).unwrap();
        assert_eq!(mailbox.take_or_retire(&first), Some(2));
        assert_eq!(mailbox.take_or_retire(&first), None);
        assert!(first.recv().is_err());
        mailbox.send_if_running(0);
        let mut receive = None;
        mailbox
            .send(3, |r| {
                receive = Some(r);
                Ok(())
            })
            .unwrap();
        assert_eq!(receive.unwrap().recv().unwrap(), 3);
    }

    #[test]
    fn a_failed_start_can_be_retried() {
        let mailbox = WorkerMailbox::new();
        assert!(mailbox
            .send(1, |_| Err(std::io::Error::other("cannot spawn")))
            .is_err());
        let mut receive = None;
        mailbox
            .send(2, |r| {
                receive = Some(r);
                Ok(())
            })
            .unwrap();
        assert_eq!(receive.unwrap().recv().unwrap(), 2);
    }

    #[test]
    fn a_dead_worker_is_replaced() {
        let mailbox = WorkerMailbox::new();
        let mut receive = None;
        mailbox
            .send(1, |r| {
                receive = Some(r);
                Ok(())
            })
            .unwrap();
        drop(receive);
        let mut receive = None;
        mailbox
            .send(2, |r| {
                receive = Some(r);
                Ok(())
            })
            .unwrap();
        assert_eq!(receive.unwrap().recv().unwrap(), 2);
    }

    #[test]
    fn sending_during_retirement_keeps_every_command() {
        let mailbox = Arc::new(WorkerMailbox::new());
        let (finished, receive) = mpsc::channel();
        for command in 0..200 {
            let owner = mailbox.clone();
            let finished = finished.clone();
            mailbox
                .send(command, move |commands| {
                    std::thread::spawn(move || loop {
                        match commands.recv_timeout(Duration::ZERO) {
                            Ok(command) => finished.send(command).unwrap(),
                            Err(_) => match owner.take_or_retire(&commands) {
                                Some(command) => finished.send(command).unwrap(),
                                None => return,
                            },
                        }
                    });
                    Ok(())
                })
                .unwrap();
        }
        let mut got: Vec<_> = (0..200)
            .map(|_| receive.recv_timeout(Duration::from_secs(5)).unwrap())
            .collect();
        got.sort_unstable();
        assert_eq!(got, (0..200).collect::<Vec<_>>());
    }
}
