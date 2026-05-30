//! `AsyncEvent` — a tiny set / clear / is_set / wait event primitive.

use std::sync::Arc;

use tokio::sync::watch;

/// Built on a `watch` channel so a `set()` that happens between two `wait()`
/// calls is never lost (unlike a bare `Notify`).
#[derive(Clone)]
pub(crate) struct AsyncEvent {
    tx: Arc<watch::Sender<bool>>,
}

impl AsyncEvent {
    pub(crate) fn new() -> Self {
        let (tx, _rx) = watch::channel(false);
        Self { tx: Arc::new(tx) }
    }

    pub(crate) fn set(&self) {
        self.tx.send_replace(true);
    }

    pub(crate) fn is_set(&self) -> bool {
        *self.tx.borrow()
    }

    pub(crate) async fn wait(&self) {
        let mut rx = self.tx.subscribe();
        loop {
            if *rx.borrow_and_update() {
                return;
            }
            if rx.changed().await.is_err() {
                return;
            }
        }
    }
}
