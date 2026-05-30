mod controller;
mod event;
mod protocol;
mod reporter;
mod shared;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use controller::DeskController;
pub use protocol::{arrive_tolerance_cm, cm_to_raw, raw_to_cm, Direction, DESK_NAME_PREFIX};
pub use reporter::{BluetoothState, ConnectionState, DeskInfo, DeskReporter};
