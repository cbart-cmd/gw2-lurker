use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread::{Builder, JoinHandle};

static STOPPING: AtomicBool = AtomicBool::new(false);
static HANDLES: OnceLock<Mutex<Vec<JoinHandle<()>>>> = OnceLock::new();

fn handles() -> &'static Mutex<Vec<JoinHandle<()>>> {
    HANDLES.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn spawn(name: &str, task: impl FnOnce() + Send + 'static) -> io::Result<()> {
    let mut handles = handles().lock().unwrap_or_else(|error| error.into_inner());
    if STOPPING.load(Ordering::Acquire) {
        return Err(io::Error::other("addon is shutting down"));
    }

    let mut index = 0;
    while index < handles.len() {
        if handles[index].is_finished() {
            let finished = handles.remove(index);
            log_join_failure(finished);
        } else {
            index += 1;
        }
    }

    handles.push(Builder::new().name(name.to_owned()).spawn(task)?);
    Ok(())
}

pub fn is_stopping() -> bool {
    STOPPING.load(Ordering::Acquire)
}

pub fn stop_all() {
    STOPPING.store(true, Ordering::Release);
    let handles = std::mem::take(&mut *handles().lock().unwrap_or_else(|error| error.into_inner()));
    for handle in handles {
        log_join_failure(handle);
    }
}

fn log_join_failure(handle: JoinHandle<()>) {
    let name = handle.thread().name().unwrap_or("unnamed").to_owned();
    if handle.join().is_err() {
        log::error!("background task {name} panicked");
    }
}
