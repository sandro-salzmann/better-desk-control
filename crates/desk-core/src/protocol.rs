//! The LINAK desk wire protocol: characteristic UUIDs, command bytes, the
//! drive-tuning constants, and the raw<->cm conversion. Pure and stateless.

use std::time::Duration;

use uuid::Uuid;

/// LINAK desks advertise as "Desk XXXX".
pub const DESK_NAME_PREFIX: &str = "Desk";

pub(crate) const CHAR_MOVE: Uuid = uuid::uuid!("99fa0002-338a-1024-8a49-009c0215f78a"); // write up/down/stop
pub(crate) const CHAR_REFOUT: Uuid = uuid::uuid!("99fa0021-338a-1024-8a49-009c0215f78a"); // height+speed notify
pub(crate) const CHAR_DPG: Uuid = uuid::uuid!("99fa0011-338a-1024-8a49-009c0215f78a"); // info / preset channel
pub(crate) const CHAR_REFIN: Uuid = uuid::uuid!("99fa0031-338a-1024-8a49-009c0215f78a"); // release / move-to-target

pub(crate) const CMD_UP: [u8; 2] = [0x47, 0x00];
pub(crate) const CMD_DOWN: [u8; 2] = [0x46, 0x00];
pub(crate) const CMD_STOP: [u8; 2] = [0xff, 0x00];
pub(crate) const CMD_RELEASE: [u8; 2] = [0x01, 0x80];

/// UI slot -> DPG sub-command byte.
pub(crate) fn preset_subcmd(slot: u8) -> u8 {
    match slot {
        1 => 0x89,
        2 => 0x8a,
        3 => 0x8b,
        _ => 0x89,
    }
}

pub(crate) const POLL: Duration = Duration::from_millis(200); // desk halts if we don't ping more often than ~250 ms
pub(crate) const BRAKE_LEAD: i32 = 25; // raw counts before target where we issue STOP (desk coasts that far)
pub(crate) const ARRIVE_TOL: i32 = 8; // raw counts (~0.8 mm) considered "already at target" — skip the move
pub(crate) const FINE_TOL: i32 = 4; // raw counts target precision during the fine-tune phase
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

pub(crate) fn dir_cmd(direction: &str) -> [u8; 2] {
    if direction.eq_ignore_ascii_case("down") {
        CMD_DOWN
    } else {
        CMD_UP
    }
}

pub(crate) fn hex_or(data: &[u8], default: &str) -> String {
    if data.is_empty() {
        default.to_string()
    } else {
        data.iter().map(|b| format!("{b:02x}")).collect()
    }
}
