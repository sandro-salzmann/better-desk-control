//! Save a preset and drive to one with early-brake + fine-tune.

use std::sync::Arc;

use btleplug::api::{Peripheral as _, WriteType};
use tokio::time::sleep;

use super::DeskController;
use crate::event::AsyncEvent;
use crate::protocol::{
    hex_or, preset_subcmd, raw_to_cm, ARRIVE_TOL, CMD_DOWN, CMD_RELEASE, CMD_STOP, CMD_UP, FINE_MAX,
    FINE_PULSE, FINE_SETTLE, FINE_TOL, POLL,
};
use crate::shared::ArriveTarget;

impl DeskController {
    /// Save the current height into a preset slot.
    pub async fn save_preset(&self, slot: u8) {
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
                self.emit_status("no height yet");
                return;
            }
        };
        let sub = preset_subcmd(slot);
        let mut payload = vec![0x7f, sub, 0x80, 0x01];
        payload.extend_from_slice(&(height as u16).to_le_bytes());
        let rsp = self.dpg(&conn, &payload).await;
        if rsp.starts_with(&[0x01, 0x00]) {
            self.emit_status(format!("saved preset {slot} @ {:.1} cm", raw_to_cm(height)));
        } else {
            self.emit_status(format!("save {slot} reply: {}", hex_or(&rsp, "(none)")));
        }
    }

    /// Start a move-to-preset that runs in the background (event-driven UIs).
    pub async fn apply_preset_cmd(self: &Arc<Self>, slot: u8) {
        self.spawn_busy(move |this, ev| async move { this.apply_preset(slot, ev).await })
            .await;
    }

    /// Move to a preset and block until the desk arrives.
    pub async fn move_to_preset(&self, slot: u8) {
        // a stop event that is never set — the move runs to completion
        self.apply_preset(slot, AsyncEvent::new()).await;
    }

    /// Drive to the preset height with early-brake + fine-tune.
    async fn apply_preset(&self, slot: u8, stop_event: AsyncEvent) {
        let conn = match self.conn.lock().await.clone() {
            Some(c) => c,
            None => {
                self.emit_status("not connected");
                return;
            }
        };
        let sub = preset_subcmd(slot);
        let rsp = self.dpg(&conn, &[0x7f, sub, 0x00]).await;
        if rsp.len() < 5 || rsp[0] != 0x01 {
            self.emit_status(format!("preset {slot} empty ({})", hex_or(&rsp, "no reply")));
            return;
        }
        let target = u16::from_le_bytes([rsp[3], rsp[4]]) as i32;
        let height = match *self.shared.height.lock().unwrap() {
            Some(h) => h,
            None => {
                self.emit_status("no current height");
                return;
            }
        };
        if (height - target).abs() <= ARRIVE_TOL {
            self.emit_status(format!("already at {target} (h={height})"));
            return;
        }
        let going_up = height < target;
        let cmd = if going_up { CMD_UP } else { CMD_DOWN };
        self.emit_status(format!(
            "→ preset {target} ({})",
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

        // fine-tune: short pulses until we're within FINE_TOL of target
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
            if diff.abs() <= FINE_TOL {
                break;
            }
            let pulse = if diff > 0 { CMD_UP } else { CMD_DOWN };
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
                .write(&conn.move_c, &CMD_STOP, WriteType::WithoutResponse)
                .await
                .is_err()
            {
                self.emit_status("err: pulse stop failed");
                break;
            }
        }
        let _ = conn
            .peripheral
            .write(&conn.refin_c, &CMD_RELEASE, WriteType::WithoutResponse)
            .await;
        let final_h = *self.shared.height.lock().unwrap();
        self.emit_status(format!(
            "arrived @ {}",
            final_h.map(|v| v.to_string()).unwrap_or_default()
        ));
    }
}
