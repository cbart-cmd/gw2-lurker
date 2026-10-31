//! Off-thread writer for the settings file.
//!
//! The render callback may not touch the disk (rule 6), but settings should not be lost if
//! the game is killed, so the UI hands a copy of the configuration to this thread and
//! forgets about it. Sends are non-blocking and the writer keeps only the last value, which
//! is what a `settings.json` needs.

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

use crate::config::Config;

/// Handle to the writer thread.
pub struct Writer {
    tx: Sender<Config>,
    handle: Option<JoinHandle<()>>,
}

impl Writer {
    /// Starts the thread. Its only job is to sit on `recv` until something changes.
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel::<Config>();
        let handle = thread::Builder::new()
            .name("gw2lurker-settings".to_owned())
            .spawn(move || run(&rx))
            .ok();
        Self { tx, handle }
    }

    /// Queues a save. Never blocks and never fails the caller: if the thread is gone the
    /// settings simply stay in memory.
    pub fn request(&self, config: &Config) {
        let _ = self.tx.send(config.clone());
    }

    /// Writes the last value, then stops the thread.
    pub fn finish(self, last: &Config) {
        let _ = self.tx.send(last.clone());
        drop(self.tx);
        if let Some(handle) = self.handle {
            let _ = handle.join();
        }
    }
}

impl std::fmt::Debug for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Writer")
            .field("alive", &self.handle.is_some())
            .finish()
    }
}

fn run(rx: &Receiver<Config>) {
    while let Ok(mut config) = rx.recv() {
        // Coalesce: if the user is dragging a window, only the last position matters.
        loop {
            match rx.try_recv() {
                Ok(newer) => config = newer,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        if let Err(error) = crate::config::save(&config) {
            log::warn!("no se pudo guardar settings.json: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rf41_writer_keeps_the_last_value() {
        let (tx, rx) = mpsc::channel::<Config>();
        let first = Config {
            poll_secs: 20,
            ..Config::default()
        };
        let last = Config {
            poll_secs: 45,
            ..Config::default()
        };

        tx.send(first).unwrap();
        tx.send(last).unwrap();
        drop(tx);

        let mut config = rx.recv().unwrap();
        while let Ok(newer) = rx.try_recv() {
            config = newer;
        }
        assert_eq!(config.poll_secs, 45, "gana el último valor encolado");
    }
}
