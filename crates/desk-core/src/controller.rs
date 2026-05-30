//! `DeskController`: all BLE work lives here.
//!
//! The struct, its lifecycle, and the cheap accessors live in this module
//! root; the behaviour is split across child modules, each holding one
//! `impl DeskController` block:
//!
//! * [`connection`]: discovery, connect/disconnect, characteristic setup.
//! * [`command`]: the low-level move and `stop` primitives.
//! * [`hold`]: press-and-hold up/down moves and the "busy" task machinery.
//! * [`preset`]: drive to an arbitrary raw target (early-brake + fine-tune).

mod command;
mod connection;
mod hold;
mod preset;

use std::sync::Arc;

use btleplug::api::Characteristic;
use btleplug::platform::Peripheral;
use tokio::sync::Mutex;

use crate::event::AsyncEvent;
use crate::protocol::raw_to_cm;
use crate::reporter::DeskReporter;
use crate::shared::Shared;

/// The connected peripheral plus the characteristics we use. Cheap to clone
/// (btleplug handles), so move loops snapshot it without holding the
/// connection lock.
#[derive(Clone)]
struct Conn {
    peripheral: Peripheral,
    move_c: Characteristic,
    refout_c: Characteristic,
    refin_c: Characteristic,
}

pub struct DeskController {
    conn: Mutex<Option<Conn>>,
    /// Set while an up/down hold or move-to-height is running.
    busy: Mutex<Option<AsyncEvent>>,
    /// Handle to the streaming-scan background task, if one is running.
    scan_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    shared: Arc<Shared>,
}

impl DeskController {
    pub fn new(reporter: Arc<dyn DeskReporter>) -> Self {
        Self {
            conn: Mutex::new(None),
            busy: Mutex::new(None),
            scan_task: Mutex::new(None),
            shared: Arc::new(Shared::new(reporter)),
        }
    }

    fn emit_status(&self, msg: impl AsRef<str>) {
        self.shared.status(msg.as_ref());
    }

    /// Most recent height in cm, or `None` if unknown.
    pub fn current_cm(&self) -> Option<f64> {
        self.shared.height.lock().unwrap().map(raw_to_cm)
    }

    /// Most recent raw height count, or `None` if unknown.
    pub fn current_raw(&self) -> Option<i32> {
        *self.shared.height.lock().unwrap()
    }

    pub async fn is_connected(&self) -> bool {
        self.conn.lock().await.is_some()
    }

    /// Whether a background move/hold is currently running.
    pub async fn is_busy(&self) -> bool {
        self.busy.lock().await.is_some()
    }
}
