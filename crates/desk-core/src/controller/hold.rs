//! Press-and-hold up/down moves and the single-background-task ("busy")
//! machinery they share with apply-preset.

use std::sync::Arc;
use std::time::Duration;

use btleplug::api::{Peripheral as _, WriteType};
use tokio::sync::watch;
use tokio::time::{sleep, timeout};

use super::DeskController;
use crate::event::AsyncEvent;
use crate::protocol::{Direction, POLL};

/// Refuse a hold-to-target move for a gap smaller than this many raw counts
/// (1 raw ≈ 0.1 mm): below it the desk can't meaningfully reposition, so a
/// "move" would just be jitter. Also the bootstrap stop point on the very first
/// move, before the coast model is calibrated.
const TARGET_DEADBAND: i32 = 10;

/// Hard cap on the predicted coast distance (raw counts) we'll lead the target
/// by, so a bad `factor` (e.g. learned from a near-stationary cutoff) can never
/// stop the desk catastrophically early.
const MAX_LEAD: i32 = 120;

/// How long to keep watching height after STOP for the desk to coast to rest,
/// so we can measure the real overshoot. Bounded so a missing notification
/// can't wedge the move task (and thus the busy flag).
const SETTLE_TIMEOUT: Duration = Duration::from_millis(800);

/// EMA weight for folding a freshly measured coast factor into the model.
const LEAD_ALPHA: f64 = 0.3;

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
        self.spawn_busy(direction, move |this, ev| async move {
            this.hold(cmd, ev).await
        })
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

    /// Like [`hold`](Self::hold), but stops on its own once the desk will *coast*
    /// onto `target_raw`, instead of running until the finger lifts.
    ///
    /// The desk overshoots a fixed stop point because the motor coasts after we
    /// cut it, and that coast grows with speed. So rather than stopping when the
    /// live height reaches the target, we cut the motor early by the predicted
    /// coast — `speed × factor` — so the coast lands us on the target. `factor`
    /// is learned per direction from the coast we actually observe (see
    /// [`learn_coast`](Self::learn_coast)); the first move, before it's
    /// calibrated, falls back to the fixed [`TARGET_DEADBAND`].
    ///
    /// We wake on every fresh height/speed notification (~20×/s) so the cutoff
    /// fires the instant the prediction crosses the target, falling back to
    /// [`POLL`] so the desk's dead-man timer never expires.
    async fn drive_to(&self, direction: Direction, target_raw: i32, stop_event: AsyncEvent) {
        let cmd = direction.command();
        let mut heights = self.shared.height_updates();
        let factor = match direction {
            Direction::Up => self.lead.lock().unwrap().up,
            Direction::Down => self.lead.lock().unwrap().down,
        };
        // Height and speed at the instant we cut the motor, for calibration.
        // Stays `None` if the finger lifts (not a clean arrival to learn from).
        let mut cutoff: Option<(i32, i32)> = None;

        loop {
            if stop_event.is_set() {
                break;
            }
            if let Some(current) = self.current_raw() {
                let speed = self.current_speed().unwrap_or(0);
                let reached = match factor {
                    // Calibrated: stop when the *projected* resting height (after
                    // coasting `speed × factor`) reaches the target.
                    Some(f) => {
                        let lead = ((speed as f64 * f).round() as i32).clamp(-MAX_LEAD, MAX_LEAD);
                        match direction {
                            Direction::Up => current + lead >= target_raw,
                            Direction::Down => current + lead <= target_raw,
                        }
                    }
                    // Uncalibrated bootstrap: the old fixed-deadband stop point.
                    None => match direction {
                        Direction::Up => current >= target_raw - TARGET_DEADBAND,
                        Direction::Down => current <= target_raw + TARGET_DEADBAND,
                    },
                };
                if reached {
                    cutoff = Some((current, speed));
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
            tokio::select! {
                _ = stop_event.wait() => {}
                _ = heights.changed() => {}
                _ = sleep(POLL) => {}
            }
        }
        self.stop().await;

        // Watch the coast settle and fold the real overshoot back into the model.
        if let Some((cutoff_h, cutoff_v)) = cutoff {
            if let Some(rest) = self.wait_until_settled(&mut heights).await {
                self.learn_coast(direction, cutoff_h, cutoff_v, rest);
            }
        }
    }

    /// After STOP, watch height until the desk reports it has stopped (or a
    /// short timeout), and return its resting height.
    async fn wait_until_settled(&self, heights: &mut watch::Receiver<u64>) -> Option<i32> {
        let settle = async {
            loop {
                if self.current_speed() == Some(0) {
                    return self.current_raw();
                }
                if heights.changed().await.is_err() {
                    return self.current_raw();
                }
            }
        };
        // On timeout the last reading is the resting height (notifications stop
        // once the desk is idle), so fall back to it either way.
        (timeout(SETTLE_TIMEOUT, settle).await).unwrap_or_else(|_| self.current_raw())
    }

    /// Fold one observed move into the coast model: the ratio of how far the
    /// desk coasted past the cutoff to the speed it carried there is the
    /// per-unit-speed lead `factor`, smoothed by an EMA. Implausible samples
    /// (wrong-sign coast, near-stationary cutoff, outliers) are dropped so they
    /// can't poison the model.
    fn learn_coast(&self, direction: Direction, cutoff_h: i32, cutoff_v: i32, rest: i32) {
        let coast = rest - cutoff_h;
        let sane = match direction {
            Direction::Up => coast >= 0 && cutoff_v > 2,
            Direction::Down => coast <= 0 && cutoff_v < -2,
        };
        if !sane || coast.abs() > MAX_LEAD {
            return;
        }
        let observed = coast as f64 / cutoff_v as f64;
        let model = {
            let mut lead = self.lead.lock().unwrap();
            let slot = match direction {
                Direction::Up => &mut lead.up,
                Direction::Down => &mut lead.down,
            };
            *slot = Some(match *slot {
                Some(prev) => prev * (1.0 - LEAD_ALPHA) + observed * LEAD_ALPHA,
                None => observed,
            });
            *lead
        };
        // Let the host persist the updated calibration.
        self.shared.calibration(model);
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
