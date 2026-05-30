//! How the controller surfaces status, connection, motion, and height to the
//! caller.

use serde::{Deserialize, Serialize};

/// High-level connection lifecycle, reported as it changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
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

/// Sink for the controller's status messages, lifecycle changes, and height
/// updates. The Tauri app emits these as window events; the CLI prints them.
///
/// Only [`status`](DeskReporter::status) and [`height`](DeskReporter::height)
/// are required; the structured callbacks default to no-ops so simple
/// consumers (e.g. the CLI) can ignore them.
pub trait DeskReporter: Send + Sync + 'static {
    fn status(&self, msg: &str);
    fn height(&self, raw: i32, cm: f64);

    /// Connection lifecycle changed. `name` is the desk's advertised name when
    /// known.
    fn connection(&self, _state: ConnectionState, _name: Option<&str>) {}

    /// A background move/hold started (`true`) or finished (`false`).
    fn motion(&self, _moving: bool) {}

    /// A matching desk was seen during a streaming scan.
    fn discovered(&self, _desk: &DeskInfo) {}

    /// The Bluetooth adapter's availability changed (e.g. the radio was toggled
    /// off/on while running).
    fn bluetooth(&self, _state: BluetoothState) {}
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
