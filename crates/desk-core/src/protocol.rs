//! The LINAK desk wire protocol: characteristic UUIDs, command bytes, the
//! drive-tuning constants, and the raw<->cm conversion. Pure and stateless.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// LINAK desks advertise as "Desk XXXX".
pub const DESK_NAME_PREFIX: &str = "Desk";

pub(crate) const CHARACTERISTIC_MOVE: Uuid = uuid::uuid!("99fa0002-338a-1024-8a49-009c0215f78a"); // write up/down/stop
pub(crate) const CHARACTERISTIC_REFERENCE_OUT: Uuid = uuid::uuid!("99fa0021-338a-1024-8a49-009c0215f78a"); // height+speed notify
pub(crate) const CHARACTERISTIC_REFERENCE_IN: Uuid = uuid::uuid!("99fa0031-338a-1024-8a49-009c0215f78a"); // release / move-to-target

pub(crate) const COMMAND_UP: [u8; 2] = [0x47, 0x00];
pub(crate) const COMMAND_DOWN: [u8; 2] = [0x46, 0x00];
pub(crate) const COMMAND_STOP: [u8; 2] = [0xff, 0x00];
pub(crate) const COMMAND_RELEASE: [u8; 2] = [0x01, 0x80];

pub(crate) const POLL: Duration = Duration::from_millis(200); // desk halts if we don't ping more often than ~250 ms
pub(crate) const BRAKE_LEAD: i32 = 25; // raw counts before target where we issue STOP (desk coasts that far)
pub(crate) const ARRIVE_TOLERANCE: i32 = 8; // raw counts (~0.8 mm) considered "already at target", skip the move
pub(crate) const FINE_TOLERANCE: i32 = 4; // raw counts target precision during the fine-tune phase
pub(crate) const FINE_PULSE: Duration = Duration::from_millis(80); // duration of a single nudge pulse
pub(crate) const FINE_SETTLE: Duration = Duration::from_millis(400); // wait after a pulse so the reading stabilizes
pub(crate) const FINE_MAX: u32 = 8; // safety cap on number of fine-tune iterations

// raw count -> height. The LINAK convention is cm = raw/100 + base_cm.
// If your physical reading disagrees, just shift HEIGHT_BASE_CM.
const HEIGHT_BASE_CM: f64 = 62.0;

/// Convert a raw height count to centimetres.
pub fn raw_to_cm(raw: i32) -> f64 {
    raw as f64 / 100.0 + HEIGHT_BASE_CM
}

/// Convert a height in centimetres to a raw count (inverse of [`raw_to_cm`]).
/// The frontend always works in cm; this is where cm targets become raw.
pub fn cm_to_raw(cm: f64) -> i32 {
    ((cm - HEIGHT_BASE_CM) * 100.0).round() as i32
}

/// The arrival tolerance ([`ARRIVE_TOLERANCE`]) expressed in centimetres (raw
/// counts are cm/100). This is the single source of truth for "is the desk at
/// this target?": the controller uses the raw form to skip a no-op move, the
/// frontend uses this cm form to flag the matching preset, so the two values
/// can never silently drift apart.
pub fn arrive_tolerance_cm() -> f64 {
    ARRIVE_TOLERANCE as f64 / 100.0
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
