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
    ARRIVE_TOLERANCE, COMMAND_DOWN, COMMAND_RELEASE, COMMAND_STOP, COMMAND_UP, Direction, FINE_MAX,
    FINE_PULSE, FINE_SETTLE, FINE_TOLERANCE, POLL,
};
use crate::shared::ArriveTarget;

impl DeskController {
    /// Drive to `target` (raw counts) as the single background "busy" task,
    /// returning immediately (event-driven UIs). Stoppable via
    /// [`stop_busy`](Self::stop_busy). No-op if the current height isn't known
    /// yet, since the drive direction is derived from it (and `drive_to_target`
    /// would short-circuit anyway).
    pub async fn move_to_height_cmd(self: &Arc<Self>, target: i32) {
        let Some(cur) = self.current_raw() else {
            return;
        };
        let direction = if cur < target {
            Direction::Up
        } else {
            Direction::Down
        };
        self.spawn_busy(direction, move |this, ev| async move {
            this.drive_to_target(target, ev).await
        })
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
            None => return,
        };
        let height = match *self.shared.height.lock().unwrap() {
            Some(h) => h,
            None => return,
        };
        if (height - target).abs() <= ARRIVE_TOLERANCE {
            return;
        }
        let going_up = height < target;
        let cmd = if going_up { COMMAND_UP } else { COMMAND_DOWN };

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
            if conn
                .peripheral
                .write(&conn.move_c, &cmd, WriteType::WithoutResponse)
                .await
                .is_err()
            {
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
        for _ in 0..FINE_MAX {
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
            if conn
                .peripheral
                .write(&conn.move_c, &pulse, WriteType::WithoutResponse)
                .await
                .is_err()
            {
                break;
            }
            sleep(FINE_PULSE).await;
            if conn
                .peripheral
                .write(&conn.move_c, &COMMAND_STOP, WriteType::WithoutResponse)
                .await
                .is_err()
            {
                break;
            }
        }
        let _ = conn
            .peripheral
            .write(&conn.refin_c, &COMMAND_RELEASE, WriteType::WithoutResponse)
            .await;
    }
}
