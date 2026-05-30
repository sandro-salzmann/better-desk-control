//! How the controller surfaces connection, motion, and height to the caller.

use serde::{Deserialize, Serialize};

use crate::protocol::Direction;

/// High-level connection lifecycle, reported as it changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

impl ConnectionState {
    /// Stable wire string; mirrored by the frontend.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disconnected => "disconnected",
            Self::Connecting => "connecting",
            Self::Connected => "connected",
        }
    }
}

/// Bluetooth adapter availability, reported at startup and on every change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BluetoothState {
    /// Adapter present and powered on.
    Ready,
    /// Bluetooth is unavailable: either the radio is switched off or no adapter
    /// is present. Both cases look the same to the user, so we don't split them.
    Off,
}

impl BluetoothState {
    /// Stable wire string; mirrored by the frontend.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Off => "off",
        }
    }
}

/// Which screen the app should show. Derived from `(connection, bluetooth)` by
/// the controller and reported via [`DeskReporter::screen`] so the frontend
/// never reproduces the decision in its own switch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Linked to a desk.
    Connected,
    /// Bluetooth is on and a connect attempt is in flight.
    Connecting,
    /// Bluetooth radio is off (or no adapter).
    BluetoothOff,
    /// Bluetooth is on but no connect is running: scan for desks.
    Scanning,
}

impl Screen {
    /// Stable wire string; mirrored by the frontend.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Connecting => "connecting",
            Self::BluetoothOff => "bluetooth_off",
            Self::Scanning => "scanning",
        }
    }
}

/// Sink for the controller's lifecycle changes and height updates. The Tauri
/// app emits these as window events; the CLI prints them.
///
/// Only [`height`](DeskReporter::height) is required; the structured callbacks
/// default to no-ops so simple consumers (e.g. the CLI) can ignore them.
pub trait DeskReporter: Send + Sync + 'static {
    fn height(&self, raw: i32, cm: f64);

    /// Connection lifecycle changed. `name` and `address` identify the desk
    /// when known (set during `Connecting` and `Connected`, cleared on
    /// `Disconnected`).
    fn connection(&self, _state: ConnectionState, _name: Option<&str>, _address: Option<&str>) {}

    /// A background move/hold started or finished. `direction` is set while
    /// `moving` is true and indicates which way the desk is being driven.
    fn motion(&self, _moving: bool, _direction: Option<Direction>) {}

    /// A matching desk was seen during a streaming scan.
    fn discovered(&self, _desk: &DeskInfo) {}

    /// The Bluetooth adapter's availability changed (e.g. the radio was toggled
    /// off/on while running).
    fn bluetooth(&self, _state: BluetoothState) {}

    /// The screen the app should show changed. Derived inside the controller
    /// from `(connection, bluetooth)`; emitted whenever either of those flips
    /// so the UI doesn't redo the derivation.
    fn screen(&self, _screen: Screen) {}
}

/// A discovered desk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeskInfo {
    pub name: String,
    pub address: String,
    /// Signal strength in dBm, when the adapter reports it.
    #[serde(default)]
    pub rssi: Option<i16>,
}
