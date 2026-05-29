mod controller;
mod event;
mod protocol;
mod reporter;
mod shared;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use controller::DeskController;
pub use protocol::{raw_to_cm, DESK_NAME_PREFIX};
pub use reporter::{DeskInfo, DeskReporter};
