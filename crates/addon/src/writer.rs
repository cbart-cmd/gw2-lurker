//! Off-thread writer for the settings file.
//!
//! The render callback may not touch the disk (rule 6), but settings should not be lost if
//! the game is killed, so the UI hands a copy of the configuration to this thread and
//! forgets about it. Sends are non-blocking and the writer keeps only the last value, which
//! is what a `settings.json` needs.

use std::io;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

use crate::config::Config;

/// Handle to the writer thread.
pub struct Writer {
    tx: Sender<Config>,
    handle: JoinHandle<()>,
}

impl Writer {
    /// Starts the thread. Its only job is to sit on `recv` until something changes.
    pub fn start() -> io::Result<Self> {
        let (tx, rx) = mpsc::channel::<Config>();
        let handle = thread::Builder::new()
            .name("gw2lurker-settings".to_owned())
            .spawn(move || run(&rx))?;
        Ok(Self { tx, handle })
    }

    /// Queues a save without blocking the caller.
    pub fn request(&self, config: &Config) -> bool {
        self.tx.send(config.clone()).is_ok()
    }

    /// Writes the last value, then stops the thread.
    pub fn finish(self, last: &Config) {
        let sent = self.tx.send(last.clone()).is_ok();
        drop(self.tx);
        let joined = self.handle.join().is_ok();
        if !sent || !joined {
            log::error!("settings writer stopped unexpectedly; saving the final settings directly");
            if let Err(error) = crate::config::save(last) {
                log::error!("no se pudo guardar settings.json al cerrar: {error}");
            }
        }
    }
}

impl std::fmt::Debug for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Writer")
            .field("alive", &!self.handle.is_finished())
            .finish()
    }
}

fn run(rx: &Receiver<Config>) {
    run_with(rx, crate::config::save);
}

fn run_with(rx: &Receiver<Config>, mut save: impl FnMut(&Config) -> io::Result<()>) {
    while let Ok(mut config) = rx.recv() {
        // Coalesce: if the user is dragging a window, only the last position matters.
        loop {
            match rx.try_recv() {
                Ok(newer) => config = newer,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        if let Err(error) = save(&config) {
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

        let mut saved = Vec::new();
        run_with(&rx, |config| {
            saved.push(config.clone());
            Ok(())
        });

        assert_eq!(saved.len(), 1, "las solicitudes pendientes se agrupan");
        assert_eq!(saved[0].poll_secs, 45, "gana el último valor encolado");
    }
}
