//! Polling for evented actor types.

#[cfg(test)]
mod tests {
    use super::*;
    use zmq;

    #[test]
    fn default_poller_has_no_pre_allocation() {
        let poller: Poller = Default::default();
        assert_eq!(poller.actors.capacity(), 0);
    }

    #[test]
    fn new_poller_can_be_created_with_capacity() {
        let poller: Poller = Poller::with_capacity(20);
        assert_eq!(poller.actors.capacity(), 20);
    }

    #[test]
    fn new_poller_can_be_created_with_existing_context_and_capacity() {
        let context = zmq::Context::new();
        let ctx = context.clone();
        let poller: Poller = Poller::with_context_and_capacity(ctx, 30);
        assert_eq!(poller.actors.capacity(), 30);
    }
}
