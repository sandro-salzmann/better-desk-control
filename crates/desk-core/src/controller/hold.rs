//! Press-and-hold up/down moves and the single-background-task ("busy")
//! machinery they share with apply-preset.

use std::sync::Arc;
use std::time::Duration;

use btleplug::api::{Peripheral as _, WriteType};
use tokio::time::{sleep, timeout};

use super::DeskController;
use crate::event::AsyncEvent;
use crate::protocol::{Direction, POLL};

impl DeskController {
    /// Spawn `work` as the single background "busy" task: a no-op if not
    /// connected or already busy, otherwise it runs `work(self, stop_event)` and
    /// clears `busy` when it returns. `stop_event` is what [`stop_busy`] trips.
    ///
    /// [`stop_busy`]: Self::stop_busy
    pub(super) async fn spawn_busy<F, Fut>(self: &Arc<Self>, work: F)
    where
        F: FnOnce(Arc<Self>, AsyncEvent) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        if self.conn.lock().await.is_none() {
            return;
        }
        let mut busy = self.busy.lock().await;
        if busy.is_some() {
            return;
        }
        let ev = AsyncEvent::new();
        *busy = Some(ev.clone());
        drop(busy);
        self.shared.motion(true);

        let this = self.clone();
        tokio::spawn(async move {
            work(this.clone(), ev).await;
            *this.busy.lock().await = None;
            this.shared.motion(false);
        });
    }

    /// Start a press-and-hold move that runs until [`stop_busy`](Self::stop_busy)
    /// (event-driven UIs).
    pub async fn start_hold(self: &Arc<Self>, direction: Direction) {
        let cmd = direction.command();
        self.spawn_busy(move |this, ev| async move { this.hold(cmd, ev).await })
            .await;
    }

    /// Hold `direction` for a fixed duration, then stop (CLI `up`/`down`).
    pub async fn hold_for(&self, direction: Direction, dur: Duration) {
        if self.conn.lock().await.is_none() {
            self.emit_status("not connected");
            return;
        }
        let cmd = direction.command();
        let stop = AsyncEvent::new();
        let timer = stop.clone();
        tokio::spawn(async move {
            sleep(dur).await;
            timer.set();
        });
        self.hold(cmd, stop).await;
    }

    /// Keep poking the move characteristic until stopped.
    async fn hold(&self, cmd: [u8; 2], stop_event: AsyncEvent) {
        loop {
            if stop_event.is_set() {
                break;
            }
            let conn = match self.conn.lock().await.clone() {
                Some(c) => c,
                None => break,
            };
            if let Err(e) = conn
                .peripheral
                .write(&conn.move_c, &cmd, WriteType::WithoutResponse)
                .await
            {
                self.emit_status(format!("err: {e}"));
                break;
            }
            let _ = timeout(POLL, stop_event.wait()).await;
        }
        self.stop().await;
    }

    /// Stop a running move, or just make sure the desk is stopped.
    pub async fn stop_busy(self: &Arc<Self>) {
        let ev = self.busy.lock().await.clone();
        match ev {
            Some(ev) => ev.set(),
            None => self.stop().await,
        }
    }
}
