//! Discovery, connect/disconnect, and characteristic setup.

use std::sync::Arc;
use std::time::Duration;

use btleplug::api::{
    Central, CentralEvent, CentralState, Characteristic, Manager as _, Peripheral as _, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use tokio::time::sleep;
use uuid::Uuid;

use super::{Conn, DeskController};
use crate::protocol::{
    CHARACTERISTIC_MOVE, CHARACTERISTIC_REFERENCE_IN, CHARACTERISTIC_REFERENCE_OUT, DESK_NAME_PREFIX,
};
use crate::reporter::{BluetoothState, ConnectionState, DeskInfo};
use crate::Result;

/// Map btleplug's adapter power state to our [`BluetoothState`]. An adapter that
/// is present but switched off reports `PoweredOff` (our `Off`); the
/// absent-hardware case is also `Off`, handled by the callers (a failed
/// `central()`), so anything else is `Ready`.
fn map_central_state(state: CentralState) -> BluetoothState {
    match state {
        CentralState::PoweredOff => BluetoothState::Off,
        _ => BluetoothState::Ready,
    }
}

impl DeskController {
    async fn central(&self) -> Result<Adapter> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        adapters
            .into_iter()
            .next()
            .ok_or_else(|| "no Bluetooth adapter found".into())
    }

    /// Current Bluetooth availability: `Ready` when an adapter is present and
    /// powered on, otherwise `Off` (radio switched off or no adapter at all).
    pub async fn bluetooth_state(&self) -> BluetoothState {
        match self.central().await {
            Ok(central) => central
                .adapter_state()
                .await
                .map(map_central_state)
                .unwrap_or(BluetoothState::Ready),
            Err(_) => BluetoothState::Off,
        }
    }

    /// Long-lived task: report the adapter's power state now and on every change,
    /// so the UI reacts when Bluetooth is toggled off/on while the app is open.
    ///
    /// btleplug emits [`CentralEvent::StateUpdate`] on toggle (on Windows it is
    /// wired to the OS `Radio.StateChanged`), which this follows. If no adapter
    /// is present yet, it reports `Off` and retries, so hardware that appears
    /// later is still picked up.
    pub async fn watch_bluetooth(self: Arc<Self>) {
        loop {
            let central = match self.central().await {
                Ok(central) => central,
                Err(_) => {
                    self.shared.bluetooth(BluetoothState::Off);
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            // emit the current state up front, then follow changes
            let now = central
                .adapter_state()
                .await
                .map(map_central_state)
                .unwrap_or(BluetoothState::Ready);
            self.shared.bluetooth(now);

            if let Ok(mut events) = central.events().await {
                while let Some(ev) = events.next().await {
                    if let CentralEvent::StateUpdate(state) = ev {
                        self.shared.bluetooth(map_central_state(state));
                    }
                }
            }

            // the event stream ended (or couldn't be opened); re-establish it
            sleep(Duration::from_secs(2)).await;
        }
    }

    /// Discover nearby desks with a single blocking scan (CLI). The streaming
    /// [`scan_start`](Self::scan_start) is what the GUI uses.
    pub async fn scan_desks(&self) -> Result<Vec<DeskInfo>> {
        self.emit_status("scanning…");
        let central = self.central().await?;
        central.start_scan(ScanFilter::default()).await?;
        sleep(Duration::from_secs(5)).await;
        central.stop_scan().await?;

        let mut out = Vec::new();
        for p in central.peripherals().await? {
            if let Some(props) = p.properties().await? {
                if let Some(name) = props.local_name {
                    if name.starts_with(DESK_NAME_PREFIX) {
                        out.push(DeskInfo {
                            name,
                            address: props.address.to_string(),
                            rssi: props.rssi,
                        });
                    }
                }
            }
        }
        Ok(out)
    }

    /// Start a streaming scan: matching desks are reported through
    /// [`DeskReporter::discovered`](crate::DeskReporter::discovered) as they are
    /// seen/updated, until [`scan_stop`](Self::scan_stop). Returns an error if
    /// the scan can't be started (e.g. the radio is off).
    pub async fn scan_start(&self) -> Result<()> {
        // make sure any previous scan task is gone first
        self.scan_stop().await;

        let central = self.central().await?;
        let events = central.events().await?;
        central.start_scan(ScanFilter::default()).await?;
        self.emit_status("scanning…");

        let central = central.clone();
        let shared = self.shared.clone();
        let handle = tokio::spawn(async move {
            let mut events = events;
            while let Some(ev) = events.next().await {
                let id = match ev {
                    CentralEvent::DeviceDiscovered(id)
                    | CentralEvent::DeviceUpdated(id)
                    | CentralEvent::ManufacturerDataAdvertisement { id, .. }
                    | CentralEvent::ServiceDataAdvertisement { id, .. } => id,
                    _ => continue,
                };
                let Ok(p) = central.peripheral(&id).await else {
                    continue;
                };
                if let Ok(Some(props)) = p.properties().await {
                    if let Some(name) = props.local_name {
                        if name.starts_with(DESK_NAME_PREFIX) {
                            shared.discovered(&DeskInfo {
                                name,
                                address: props.address.to_string(),
                                rssi: props.rssi,
                            });
                        }
                    }
                }
            }
        });
        *self.scan_task.lock().await = Some(handle);
        Ok(())
    }

    /// Stop a running streaming scan. No-op if none is running.
    pub async fn scan_stop(&self) {
        if let Some(handle) = self.scan_task.lock().await.take() {
            handle.abort();
        }
        if let Ok(central) = self.central().await {
            let _ = central.stop_scan().await;
        }
    }

    /// btleplug connects to a `Peripheral` handle, not an address string, so we
    /// scan briefly to rediscover the saved/picked address.
    async fn find_peripheral(&self, address: &str) -> Result<Option<Peripheral>> {
        let central = self.central().await?;
        central.start_scan(ScanFilter::default()).await?;
        for _ in 0..25 {
            sleep(Duration::from_millis(200)).await;
            for p in central.peripherals().await? {
                if p.address().to_string().eq_ignore_ascii_case(address) {
                    let _ = central.stop_scan().await;
                    return Ok(Some(p));
                }
            }
        }
        let _ = central.stop_scan().await;
        Ok(None)
    }

    /// Connect to a desk by address. Returns `true` on success. Remembering the
    /// address is the caller's responsibility.
    pub async fn connect(&self, address: &str) -> bool {
        // the streaming scan (if any) owns the adapter; release it before we
        // re-scan for the target address inside find_peripheral.
        self.scan_stop().await;

        self.shared.connection(ConnectionState::Connecting, None);
        self.emit_status(format!("connecting to {address}…"));
        let peripheral = match self.find_peripheral(address).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                self.emit_status("device not found");
                self.shared.connection(ConnectionState::Disconnected, None);
                return false;
            }
            Err(e) => {
                self.emit_status(format!("scan failed: {e}"));
                self.shared.connection(ConnectionState::Disconnected, None);
                return false;
            }
        };
        if let Err(e) = peripheral.connect().await {
            self.emit_status(format!("connect failed: {e}"));
            self.shared.connection(ConnectionState::Disconnected, None);
            return false;
        }
        if let Err(e) = self.setup_connection(&peripheral).await {
            self.emit_status(format!("setup failed: {e}"));
            let _ = peripheral.disconnect().await;
            self.shared.connection(ConnectionState::Disconnected, None);
            return false;
        }

        let name = peripheral
            .properties()
            .await
            .ok()
            .flatten()
            .and_then(|p| p.local_name)
            .unwrap_or_else(|| address.to_string());
        self.emit_status(format!("connected: {name}"));
        self.shared
            .connection(ConnectionState::Connected, Some(&name));
        true
    }

    async fn setup_connection(&self, peripheral: &Peripheral) -> Result<()> {
        peripheral.discover_services().await?;
        let chars = peripheral.characteristics();
        let find = |u: Uuid| -> Result<Characteristic> {
            chars
                .iter()
                .find(|c| c.uuid == u)
                .cloned()
                .ok_or_else(|| format!("missing characteristic {u}").into())
        };
        let move_c = find(CHARACTERISTIC_MOVE)?;
        let refout_c = find(CHARACTERISTIC_REFERENCE_OUT)?;
        let refin_c = find(CHARACTERISTIC_REFERENCE_IN)?;

        peripheral.subscribe(&refout_c).await?;

        // prime the height reading
        if let Ok(data) = peripheral.read(&refout_c).await {
            self.shared.on_height(&data);
        }

        // background task that fans incoming notifications to the shared state
        let mut stream = peripheral.notifications().await?;
        let shared = self.shared.clone();
        tokio::spawn(async move {
            while let Some(n) = stream.next().await {
                if n.uuid == CHARACTERISTIC_REFERENCE_OUT {
                    shared.on_height(&n.value);
                }
            }
        });

        *self.conn.lock().await = Some(Conn {
            peripheral: peripheral.clone(),
            move_c,
            refout_c,
            refin_c,
        });
        Ok(())
    }

    /// Disconnect, unsubscribe, and clear the cached height.
    pub async fn disconnect(&self) {
        let conn = self.conn.lock().await.take();
        if let Some(conn) = conn {
            if conn.peripheral.is_connected().await.unwrap_or(false) {
                let _ = conn.peripheral.unsubscribe(&conn.refout_c).await;
                let _ = conn.peripheral.disconnect().await;
            }
        }
        *self.shared.height.lock().unwrap() = None;
        self.emit_status("disconnected");
        self.shared.connection(ConnectionState::Disconnected, None);
    }
}
