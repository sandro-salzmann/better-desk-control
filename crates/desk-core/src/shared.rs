//! Shared notify state: written by the background notification task and read
//! by the command tasks. The `StdMutex` sections are short and never held
//! across an `.await`.

use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use tokio::sync::watch;

use crate::protocol::{raw_to_cm, Direction};
use crate::reporter::{BluetoothState, ConnectionState, DeskInfo, DeskReporter, LeadModel, Screen};

/// Lives behind an `Arc` so the notification-stream task can update it while
/// the command tasks read it.
pub(crate) struct Shared {
    reporter: Arc<dyn DeskReporter>,
    pub(crate) height: StdMutex<Option<i32>>,
    /// Signed speed from the same notification: positive up, negative down,
    /// `0` stopped. Drives the coast prediction in a hold-to-target move.
    pub(crate) speed: StdMutex<Option<i32>>,
    /// Bumped on every height notification (~20×/s while moving) so a
    /// hold-to-target move can re-check arrival the instant a fresh reading
    /// lands, instead of only on its ~200 ms keep-alive tick.
    height_tx: watch::Sender<u64>,
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
            speed: StdMutex::new(None),
            height_tx: watch::channel(0).0,
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

    pub(crate) fn calibration(&self, model: LeadModel) {
        self.reporter.calibration(model);
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

    /// A receiver that fires every time a fresh height notification lands. Used
    /// by a hold-to-target move to re-check arrival as fast as the desk reports,
    /// rather than waiting out its keep-alive tick.
    pub(crate) fn height_updates(&self) -> watch::Receiver<u64> {
        self.height_tx.subscribe()
    }

    /// Handle an incoming height notification.
    pub(crate) fn on_height(&self, data: &[u8]) {
        if data.len() < 2 {
            return;
        }
        let h = u16::from_le_bytes([data[0], data[1]]) as i32;
        *self.height.lock().unwrap() = Some(h);
        if data.len() >= 4 {
            *self.speed.lock().unwrap() = Some(i16::from_le_bytes([data[2], data[3]]) as i32);
        }
        self.height_tx.send_modify(|n| *n = n.wrapping_add(1));
        self.reporter.height(h, raw_to_cm(h));
    }
}
