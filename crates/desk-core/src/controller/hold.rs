//! Press-and-hold up/down moves and the single-background-task ("busy")
//! machinery they share with apply-preset.

use std::sync::Arc;
use std::time::Duration;

use btleplug::api::{Peripheral as _, WriteType};
use tokio::time::{sleep, timeout};

use super::DeskController;
use crate::event::AsyncEvent;
use crate::protocol::{Direction, POLL};

/// Stop a hold-to-target move once live height is within this many raw counts
/// of the target (1 raw ≈ 0.1 mm), and refuse to start one for a gap smaller
/// than this. Absorbs the desk's coast past the target and avoids hunting.
const TARGET_DEADBAND: i32 = 10;

impl DeskController {
    /// Spawn `work` as the single background "busy" task: a no-op if not
    /// connected or already busy, otherwise it runs `work(self, stop_event)` and
    /// clears `busy` when it returns. `stop_event` is what [`stop_busy`] trips.
    /// `direction` is reported in the motion event so the UI shows the right
    /// arrow without having to second-guess from a stale local height.
    ///
    /// [`stop_busy`]: Self::stop_busy
    pub(super) async fn spawn_busy<F, Fut>(self: &Arc<Self>, direction: Direction, work: F)
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
        self.shared.motion(true, Some(direction));

        let this = self.clone();
        tokio::spawn(async move {
            work(this.clone(), ev).await;
            *this.busy.lock().await = None;
            this.shared.motion(false, None);
        });
    }

    /// Start a press-and-hold move that runs until [`stop_busy`](Self::stop_busy)
    /// (event-driven UIs).
    pub async fn start_hold(self: &Arc<Self>, direction: Direction) {
        let cmd = direction.command();
        self.spawn_busy(direction, move |this, ev| async move { this.hold(cmd, ev).await })
            .await;
    }

    /// Start a press-and-hold move *toward* `target_raw`: drive the desk in
    /// whichever direction closes the gap and stop automatically once it's
    /// reached (or when the finger lifts, via [`stop_busy`](Self::stop_busy)).
    ///
    /// We drive directly with UP/DOWN rather than the desk's native
    /// move-to-target characteristic, which is broken on this firmware (it
    /// halts after ~5 mm — see docs/protocol.md). Deciding the direction and
    /// the stop point here keeps that control-flow in Rust; the UI only says
    /// "hold to go to this preset".
    pub async fn start_hold_to_target(self: &Arc<Self>, target_raw: i32) {
        let Some(current) = self.current_raw() else {
            return; // no height yet: nothing to aim at
        };
        // Below this gap the desk can't meaningfully reposition, so a "move"
        // would just be a jitter; treat the preset as already reached.
        if (target_raw - current).abs() <= TARGET_DEADBAND {
            return;
        }
        let direction = if target_raw > current {
            Direction::Up
        } else {
            Direction::Down
        };
        self.spawn_busy(direction, move |this, ev| async move {
            this.drive_to(direction, target_raw, ev).await
        })
        .await;
    }

    /// Hold `direction` for a fixed duration, then stop (CLI `up`/`down`).
    pub async fn hold_for(&self, direction: Direction, dur: Duration) {
        if self.conn.lock().await.is_none() {
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
            if conn
                .peripheral
                .write(&conn.move_c, &cmd, WriteType::WithoutResponse)
                .await
                .is_err()
            {
                break;
            }
            let _ = timeout(POLL, stop_event.wait()).await;
        }
        self.stop().await;
    }

    /// Like [`hold`](Self::hold), but stops as soon as the live height reaches
    /// `target_raw` in the drive direction (within [`TARGET_DEADBAND`]) instead
    /// of running until the finger lifts. Height is updated ~20×/s by the notify
    /// task while moving, so each poll sees a fresh reading.
    async fn drive_to(&self, direction: Direction, target_raw: i32, stop_event: AsyncEvent) {
        let cmd = direction.command();
        loop {
            if stop_event.is_set() {
                break;
            }
            if let Some(current) = self.current_raw() {
                let reached = match direction {
                    Direction::Up => current >= target_raw - TARGET_DEADBAND,
                    Direction::Down => current <= target_raw + TARGET_DEADBAND,
                };
                if reached {
                    break;
                }
            }
            let conn = match self.conn.lock().await.clone() {
                Some(c) => c,
                None => break,
            };
            if conn
                .peripheral
                .write(&conn.move_c, &cmd, WriteType::WithoutResponse)
                .await
                .is_err()
            {
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
