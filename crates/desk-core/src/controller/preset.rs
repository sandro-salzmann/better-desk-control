//! Drive to an arbitrary target height with early-brake + fine-tune.
//!
//! Presets are managed by the app (name + icon + target height); the controller
//! just drives to a raw target. The brake/fine-tune logic here is what makes the
//! desk land precisely on a saved height.

use std::sync::Arc;

use btleplug::api::{Peripheral as _, WriteType};
use tokio::time::sleep;

use super::DeskController;
use crate::event::AsyncEvent;
use crate::protocol::{
    raw_to_cm, ARRIVE_TOLERANCE, COMMAND_DOWN, COMMAND_RELEASE, COMMAND_STOP, COMMAND_UP, FINE_MAX,
    FINE_PULSE, FINE_SETTLE, FINE_TOLERANCE, POLL,
};
use crate::shared::ArriveTarget;

impl DeskController {
    /// Drive to `target` (raw counts) as the single background "busy" task,
    /// returning immediately (event-driven UIs). Stoppable via
    /// [`stop_busy`](Self::stop_busy).
    pub async fn move_to_height_cmd(self: &Arc<Self>, target: i32) {
        self.spawn_busy(move |this, ev| async move { this.drive_to_target(target, ev).await })
            .await;
    }

    /// Drive to `target` (raw counts) and block until the desk arrives (CLI /
    /// synchronous callers).
    pub async fn move_to_height(&self, target: i32) {
        // a stop event that is never set: the move runs to completion
        self.drive_to_target(target, AsyncEvent::new()).await;
    }

    /// Drive to the target height with early-brake + fine-tune.
    async fn drive_to_target(&self, target: i32, stop_event: AsyncEvent) {
        let conn = match self.conn.lock().await.clone() {
            Some(c) => c,
            None => {
                self.emit_status("not connected");
                return;
            }
        };
        let height = match *self.shared.height.lock().unwrap() {
            Some(h) => h,
            None => {
                self.emit_status("no current height");
                return;
            }
        };
        if (height - target).abs() <= ARRIVE_TOLERANCE {
            self.emit_status(format!("already at {target} (h={height})"));
            return;
        }
        let going_up = height < target;
        let cmd = if going_up { COMMAND_UP } else { COMMAND_DOWN };
        self.emit_status(format!(
            "→ {:.1} cm ({})",
            raw_to_cm(target),
            if going_up { "up" } else { "down" }
        ));

        // arm the notification-driven arrival check
        *self.shared.arrive_target.lock().unwrap() = Some(ArriveTarget { target, going_up });
        self.shared.arrive_event.clear();

        loop {
            if stop_event.is_set() || self.shared.arrive_event.is_set() {
                break;
            }
            if self.conn.lock().await.is_none() {
                break;
            }
            if let Err(e) = conn
                .peripheral
                .write(&conn.move_c, &cmd, WriteType::WithoutResponse)
                .await
            {
                self.emit_status(format!("err: {e}"));
                break;
            }
            // wake on stop OR arrival OR the 200ms dead-man deadline
            tokio::select! {
                _ = stop_event.wait() => {}
                _ = self.shared.arrive_event.wait() => {}
                _ = sleep(POLL) => {}
            }
        }
        *self.shared.arrive_target.lock().unwrap() = None;
        self.stop().await;

        // fine-tune: short pulses until we're within FINE_TOLERANCE of target
        for i in 0..FINE_MAX {
            if stop_event.is_set() || self.conn.lock().await.is_none() {
                break;
            }
            sleep(FINE_SETTLE).await;
            let h = match *self.shared.height.lock().unwrap() {
                Some(h) => h,
                None => break,
            };
            let diff = target - h;
            if diff.abs() <= FINE_TOLERANCE {
                break;
            }
            let pulse = if diff > 0 { COMMAND_UP } else { COMMAND_DOWN };
            self.emit_status(format!("fine {}: h={h} → {target} ({diff:+})", i + 1));
            if conn
                .peripheral
                .write(&conn.move_c, &pulse, WriteType::WithoutResponse)
                .await
                .is_err()
            {
                self.emit_status("err: pulse write failed");
                break;
            }
            sleep(FINE_PULSE).await;
            if conn
                .peripheral
                .write(&conn.move_c, &COMMAND_STOP, WriteType::WithoutResponse)
                .await
                .is_err()
            {
                self.emit_status("err: pulse stop failed");
                break;
            }
        }
        let _ = conn
            .peripheral
            .write(&conn.refin_c, &COMMAND_RELEASE, WriteType::WithoutResponse)
            .await;
        let final_h = *self.shared.height.lock().unwrap();
        self.emit_status(format!(
            "arrived @ {}",
            final_h.map(|v| v.to_string()).unwrap_or_default()
        ));
    }
}
