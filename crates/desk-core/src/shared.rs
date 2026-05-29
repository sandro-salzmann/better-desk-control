//! Shared notify state — written by the background notification task and read
//! by the command tasks. The `StdMutex` sections are short and never held
//! across an `.await`.

use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use crate::event::AsyncEvent;
use crate::protocol::{raw_to_cm, BRAKE_LEAD};
use crate::reporter::DeskReporter;

pub(crate) struct ArriveTarget {
    pub(crate) target: i32,
    pub(crate) going_up: bool,
}

/// Lives behind an `Arc` so the notification-stream task can update it while
/// the command tasks read it.
pub(crate) struct Shared {
    reporter: Arc<dyn DeskReporter>,
    pub(crate) height: StdMutex<Option<i32>>,
    /// Armed by `apply_preset`; the height-notify callback reads it to decide
    /// when the desk has reached the target and trips `arrive_event`.
    pub(crate) arrive_target: StdMutex<Option<ArriveTarget>>,
    pub(crate) arrive_event: AsyncEvent,
    pub(crate) dpg_last: StdMutex<Vec<u8>>,
    pub(crate) dpg_event: AsyncEvent,
}

impl Shared {
    pub(crate) fn new(reporter: Arc<dyn DeskReporter>) -> Self {
        Self {
            reporter,
            height: StdMutex::new(None),
            arrive_target: StdMutex::new(None),
            arrive_event: AsyncEvent::new(),
            dpg_last: StdMutex::new(Vec::new()),
            dpg_event: AsyncEvent::new(),
        }
    }

    pub(crate) fn status(&self, msg: &str) {
        self.reporter.status(msg);
    }

    /// Handle an incoming height notification.
    pub(crate) fn on_height(&self, data: &[u8]) {
        if data.len() < 2 {
            return;
        }
        let h = u16::from_le_bytes([data[0], data[1]]) as i32;
        *self.height.lock().unwrap() = Some(h);
        self.reporter.height(h, raw_to_cm(h));

        if let Some(t) = self.arrive_target.lock().unwrap().as_ref() {
            // brake early so the desk's coast lands us on target
            let brake_at = if t.going_up {
                t.target - BRAKE_LEAD
            } else {
                t.target + BRAKE_LEAD
            };
            if (t.going_up && h >= brake_at) || (!t.going_up && h <= brake_at) {
                self.arrive_event.set();
            }
        }
    }

    /// Handle an incoming DPG reply notification.
    pub(crate) fn on_dpg(&self, data: &[u8]) {
        *self.dpg_last.lock().unwrap() = data.to_vec();
        self.dpg_event.set();
    }
}
