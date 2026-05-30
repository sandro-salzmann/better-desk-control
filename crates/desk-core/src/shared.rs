//! Shared notify state: written by the background notification task and read
//! by the command tasks. The `StdMutex` sections are short and never held
//! across an `.await`.

use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use crate::event::AsyncEvent;
use crate::protocol::{raw_to_cm, Direction, BRAKE_LEAD};
use crate::reporter::{BluetoothState, ConnectionState, DeskInfo, DeskReporter, Screen};

pub(crate) struct ArriveTarget {
    pub(crate) target: i32,
    pub(crate) going_up: bool,
}

/// Lives behind an `Arc` so the notification-stream task can update it while
/// the command tasks read it.
pub(crate) struct Shared {
    reporter: Arc<dyn DeskReporter>,
    pub(crate) height: StdMutex<Option<i32>>,
    /// Armed by `drive_to_target`; the height-notify callback reads it to decide
    /// when the desk has reached the target and trips `arrive_event`.
    pub(crate) arrive_target: StdMutex<Option<ArriveTarget>>,
    pub(crate) arrive_event: AsyncEvent,
    /// Tracks the inputs that derive the current [`Screen`], so we can detect a
    /// change and emit `screen` exactly once per real transition.
    screen_state: StdMutex<ScreenState>,
}

struct ScreenState {
    connection: ConnectionState,
    bluetooth: BluetoothState,
    last_emitted: Option<Screen>,
}

impl ScreenState {
    fn derive(&self) -> Screen {
        // Connected takes precedence over a stale Off reading from the adapter.
        match self.connection {
            ConnectionState::Connected => Screen::Connected,
            ConnectionState::Connecting => match self.bluetooth {
                BluetoothState::Off => Screen::BluetoothOff,
                BluetoothState::Ready => Screen::Connecting,
            },
            ConnectionState::Disconnected => match self.bluetooth {
                BluetoothState::Off => Screen::BluetoothOff,
                BluetoothState::Ready => Screen::Scanning,
            },
        }
    }
}

impl Shared {
    pub(crate) fn new(reporter: Arc<dyn DeskReporter>) -> Self {
        Self {
            reporter,
            height: StdMutex::new(None),
            arrive_target: StdMutex::new(None),
            arrive_event: AsyncEvent::new(),
            screen_state: StdMutex::new(ScreenState {
                connection: ConnectionState::Disconnected,
                bluetooth: BluetoothState::Ready,
                last_emitted: None,
            }),
        }
    }

    pub(crate) fn connection(
        &self,
        state: ConnectionState,
        name: Option<&str>,
        address: Option<&str>,
    ) {
        self.reporter.connection(state, name, address);
        let mut s = self.screen_state.lock().unwrap();
        s.connection = state;
        self.emit_screen_if_changed(&mut s);
    }

    pub(crate) fn motion(&self, moving: bool, direction: Option<Direction>) {
        self.reporter.motion(moving, direction);
    }

    pub(crate) fn discovered(&self, desk: &DeskInfo) {
        self.reporter.discovered(desk);
    }

    pub(crate) fn bluetooth(&self, state: BluetoothState) {
        self.reporter.bluetooth(state);
        let mut s = self.screen_state.lock().unwrap();
        s.bluetooth = state;
        self.emit_screen_if_changed(&mut s);
    }

    fn emit_screen_if_changed(&self, s: &mut ScreenState) {
        let next = s.derive();
        if s.last_emitted != Some(next) {
            s.last_emitted = Some(next);
            self.reporter.screen(next);
        }
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
}
