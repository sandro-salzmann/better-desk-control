//! The LINAK desk wire protocol: characteristic UUIDs, command bytes, and the
//! raw<->cm conversion. Pure and stateless.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// LINAK desks advertise as "Desk XXXX".
pub const DESK_NAME_PREFIX: &str = "Desk";

pub(crate) const CHARACTERISTIC_MOVE: Uuid = uuid::uuid!("99fa0002-338a-1024-8a49-009c0215f78a"); // write up/down/stop
pub(crate) const CHARACTERISTIC_REFERENCE_OUT: Uuid =
    uuid::uuid!("99fa0021-338a-1024-8a49-009c0215f78a"); // height+speed notify
pub(crate) const CHARACTERISTIC_REFERENCE_IN: Uuid =
    uuid::uuid!("99fa0031-338a-1024-8a49-009c0215f78a"); // release / move-to-target

pub(crate) const COMMAND_UP: [u8; 2] = [0x47, 0x00];
pub(crate) const COMMAND_DOWN: [u8; 2] = [0x46, 0x00];
pub(crate) const COMMAND_STOP: [u8; 2] = [0xff, 0x00];
pub(crate) const COMMAND_RELEASE: [u8; 2] = [0x01, 0x80];

pub(crate) const POLL: Duration = Duration::from_millis(200); // desk halts if we don't ping more often than ~250 ms

// raw count -> height. The LINAK convention is cm = raw/100 + base_cm.
// If your physical reading disagrees, just shift HEIGHT_BASE_CM.
const HEIGHT_BASE_CM: f64 = 62.0;

/// Convert a raw height count to centimetres.
pub fn raw_to_cm(raw: i32) -> f64 {
    raw as f64 / 100.0 + HEIGHT_BASE_CM
}

/// Convert a height in centimetres back to a raw count. Inverse of
/// [`raw_to_cm`]; used to turn a saved preset (stored in cm) into the raw
/// target the move loop compares live height against.
pub fn cm_to_raw(cm: f64) -> i32 {
    ((cm - HEIGHT_BASE_CM) * 100.0).round() as i32
}

/// Which way a press-and-hold move drives the desk. Deserialized straight from
/// the Tauri command argument, so an unknown direction is a hard error rather
/// than a silent default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    /// The move-characteristic command bytes for this direction.
    pub(crate) fn command(self) -> [u8; 2] {
        match self {
            Direction::Up => COMMAND_UP,
            Direction::Down => COMMAND_DOWN,
        }
    }
}
