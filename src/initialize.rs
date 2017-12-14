pub mod errors {
    error_chain!{}
}
use self::errors::*;
use zmq;

thread_local! {
    static CTX: zmq::Context = zmq::Context::new();
}

pub fn sys_context() -> zmq::Context {
    CTX.with(|ctx| ctx.clone())
}

/// Initialize our global context. That's it for now.
pub fn init() -> Result<()> {
    Ok(())
}
