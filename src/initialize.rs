use zmq;

thread_local! {
    static CTX: zmq::Context = zmq::Context::new();
}

pub fn sys_context() -> zmq::Context {
    CTX.with(|ctx| ctx.clone())
}
