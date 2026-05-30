//! The low-level BLE command primitives every higher-level move builds on.

use btleplug::api::{Peripheral as _, WriteType};

use super::DeskController;
use crate::protocol::{COMMAND_RELEASE, COMMAND_STOP};

impl DeskController {
    /// STOP the motor and RELEASE the move-to-target latch. Halts any motion
    /// immediately (CLI `stop`).
    pub async fn stop(&self) {
        let conn = match self.conn.lock().await.clone() {
            Some(c) => c,
            None => return,
        };
        let _ = conn
            .peripheral
            .write(&conn.move_c, &COMMAND_STOP, WriteType::WithoutResponse)
            .await;
        let _ = conn
            .peripheral
            .write(&conn.refin_c, &COMMAND_RELEASE, WriteType::WithoutResponse)
            .await;
    }
}
