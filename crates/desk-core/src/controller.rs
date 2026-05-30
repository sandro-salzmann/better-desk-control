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

use std::sync::atomic::{AtomicBool, Ordering};
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
    /// Serializes [`connect_named`](Self::connect_named): one connect drives the
    /// adapter at a time. The boot guard already dedupes auto-reconnect, but a
    /// manual connect isn't boot-guarded (the scan list stays tappable while one
    /// is in flight, so a second tap fires another connect) and a Bluetooth
    /// recovery can land on top of one. Without this, the overlapping connects'
    /// scans would `stop_scan` each other; with it, latecomers find
    /// `is_connected()` and return instead of racing the adapter.
    connect_lock: Mutex<()>,
    /// Set while a one-shot startup/recovery boot owns the "what do I do now"
    /// decision. See [`try_begin_boot`](Self::try_begin_boot).
    booting: AtomicBool,
    shared: Arc<Shared>,
}

impl DeskController {
    pub fn new(reporter: Arc<dyn DeskReporter>) -> Self {
        Self {
            conn: Mutex::new(None),
            busy: Mutex::new(None),
            scan_task: Mutex::new(None),
            connect_lock: Mutex::new(()),
            booting: AtomicBool::new(false),
            shared: Arc::new(Shared::new(reporter)),
        }
    }

    /// Claim the one-shot boot decision (reconnect-to-saved vs. scan). Returns
    /// `false` if a boot is already in flight, so a duplicate trigger never
    /// starts a second connect that would bounce the UI through the scan screen.
    /// Duplicates are real: a Bluetooth off->on recovery can land on top of
    /// launch, and React StrictMode mounts the startup effect twice in dev.
    /// Pair every `true` with an [`end_boot`](Self::end_boot).
    pub fn try_begin_boot(&self) -> bool {
        self.booting
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Release the boot claimed by [`try_begin_boot`](Self::try_begin_boot).
    pub fn end_boot(&self) {
        self.booting.store(false, Ordering::Release);
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
