//! Helpful utilities.
use std::io;
use std::thread;

/// Function for spawing child-threads, returning the `thread::JoinHandle`.
pub fn run_named_thread<F, T>(
    name: &str,
    callback: F,
) -> Result<thread::JoinHandle<T>, io::Error>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    thread::Builder::new()
        .name(name.to_string())
        .spawn(callback)
}
