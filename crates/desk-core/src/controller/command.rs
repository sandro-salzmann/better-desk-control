//! The low-level BLE command primitives every higher-level move builds on.

use std::time::Duration;

use btleplug::api::{Peripheral as _, WriteType};
use tokio::time::timeout;

use super::{Conn, DeskController};
use crate::protocol::{CMD_RELEASE, CMD_STOP};

impl DeskController {
    /// Write a payload on the DPG channel and await the reply.
    pub(super) async fn dpg(&self, conn: &Conn, payload: &[u8]) -> Vec<u8> {
        self.shared.dpg_event.clear();
        if conn
            .peripheral
            .write(&conn.dpg_c, payload, WriteType::WithoutResponse)
            .await
            .is_err()
        {
            return Vec::new();
        }
        match timeout(Duration::from_secs(1), self.shared.dpg_event.wait()).await {
            Ok(_) => self.shared.dpg_last.lock().unwrap().clone(),
            Err(_) => Vec::new(),
        }
    }

    /// STOP the motor and RELEASE the move-to-target latch. Halts any motion
    /// immediately (CLI `stop`).
    pub async fn stop(&self) {
        let conn = match self.conn.lock().await.clone() {
            Some(c) => c,
            None => return,
        };
        let _ = conn
            .peripheral
            .write(&conn.move_c, &CMD_STOP, WriteType::WithoutResponse)
            .await;
        let _ = conn
            .peripheral
            .write(&conn.refin_c, &CMD_RELEASE, WriteType::WithoutResponse)
            .await;
    }
}
