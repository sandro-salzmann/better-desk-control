//! How the controller surfaces status and height to the caller.

use serde::{Deserialize, Serialize};

/// Sink for the controller's status messages and height updates. The Tauri app
/// emits these as window events; the CLI prints them.
pub trait DeskReporter: Send + Sync + 'static {
    fn status(&self, msg: &str);
    fn height(&self, raw: i32, cm: f64);
}

/// A discovered desk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeskInfo {
    pub name: String,
    pub address: String,
}
