//! Discovery, connect/disconnect, and characteristic setup.

use std::sync::Arc;
use std::time::Duration;

use btleplug::api::{
    Central, CentralEvent, CentralState, Characteristic, Manager as _, Peripheral as _, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use super::{pairing, Conn, DeskController};
use crate::protocol::{
    CHARACTERISTIC_MOVE, CHARACTERISTIC_REFERENCE_IN, CHARACTERISTIC_REFERENCE_OUT,
    DESK_NAME_PREFIX,
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

    /// Read the adapter's current power state, defaulting to `Ready` if the
    /// query fails on an otherwise-present adapter.
    async fn read_state(central: &Adapter) -> BluetoothState {
        central
            .adapter_state()
            .await
            .map(map_central_state)
            .unwrap_or(BluetoothState::Ready)
    }

    /// Current Bluetooth availability: `Ready` when an adapter is present and
    /// powered on, otherwise `Off` (radio switched off or no adapter at all).
    pub async fn bluetooth_state(&self) -> BluetoothState {
        match self.central().await {
            Ok(central) => Self::read_state(&central).await,
            Err(_) => BluetoothState::Off,
        }
    }

    /// Apply a new Bluetooth state: when the radio went off, record the `Off`
    /// before tearing down the live link, so the intermediate `(Disconnected,
    /// Ready)` derivation never happens and the screen goes straight from
    /// `Connected` to `BluetoothOff`. Remembered-desk bookkeeping is the app
    /// layer's concern; we just drop the connection.
    async fn apply_bluetooth(self: &Arc<Self>, state: BluetoothState) {
        self.shared.bluetooth(state);
        if matches!(state, BluetoothState::Off) && self.is_connected().await {
            self.disconnect().await;
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
                    self.apply_bluetooth(BluetoothState::Off).await;
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            // emit the current state up front, then follow changes
            self.apply_bluetooth(Self::read_state(&central).await).await;

            if let Ok(mut events) = central.events().await {
                while let Some(ev) = events.next().await {
                    if let CentralEvent::StateUpdate(state) = ev {
                        self.apply_bluetooth(map_central_state(state)).await;
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
    /// scan to rediscover the saved/picked desk.
    ///
    /// We follow the central's event stream (same mechanism as `scan_start`) and
    /// fetch properties for each newly-seen or updated peripheral exactly once,
    /// instead of polling every peripheral's properties on every tick. The
    /// budget is ~12s — generous because in the GUI `watch_bluetooth` shares the
    /// adapter, so discovery is slower than in the CLI.
    ///
    /// We match on the address read from [`properties`](btleplug::api::Peripheral::properties),
    /// not the `Peripheral::address()` accessor (on Windows/WinRT the latter
    /// reads back as zeros until properties are fetched).
    ///
    /// Returns the matched peripheral along with the `local_name` read during
    /// discovery, so the caller can avoid a second `properties().await`.
    async fn find_peripheral(&self, address: &str) -> Result<Option<(Peripheral, Option<String>)>> {
        let central = self.central().await?;
        // subscribe before start_scan so we don't miss events for desks that
        // were already advertising
        let mut events = central.events().await?;
        central.start_scan(ScanFilter::default()).await?;

        let found = timeout(Duration::from_secs(12), async {
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
                let Ok(Some(props)) = p.properties().await else {
                    continue;
                };
                if props.address.to_string().eq_ignore_ascii_case(address) {
                    return Some((p, props.local_name));
                }
            }
            None
        })
        .await
        .ok()
        .flatten();

        let _ = central.stop_scan().await;
        Ok(found)
    }

    /// Connect to a desk by address. Returns `true` on success. Remembering the
    /// address is the caller's responsibility.
    pub async fn connect(&self, address: &str) -> bool {
        self.connect_named(address, None).await
    }

    /// Like [`connect`](Self::connect), but the up-front `Connecting` event
    /// carries `name` so the UI can show the remembered desk's name while the
    /// link is still coming up (the live name is re-reported once connected).
    pub async fn connect_named(&self, address: &str, name: Option<&str>) -> bool {
        self.connect_named_inner(address, name, true).await
    }

    /// Like [`connect_named`](Self::connect_named) but on failure does NOT emit
    /// `Disconnected`. The UI stays on "Connecting…" between attempts; the
    /// caller (boot's retry loop) is expected to wait and try again until it
    /// either succeeds or some other path tears things down.
    pub async fn try_connect_named(&self, address: &str, name: Option<&str>) -> bool {
        self.connect_named_inner(address, name, false).await
    }

    /// Retry [`try_connect_named`](Self::try_connect_named) forever (1s backoff)
    /// until it succeeds. From the user's perspective, the connecting screen
    /// just stays up until the desk is reachable, instead of bouncing through
    /// a scan screen on a transient failure. Used by the boot reconnect path.
    pub async fn connect_named_persistent(&self, address: &str, name: Option<&str>) {
        while !self.try_connect_named(address, name).await {
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn connect_named_inner(
        &self,
        address: &str,
        name: Option<&str>,
        emit_failure: bool,
    ) -> bool {
        // Serialize connects: only one may drive the adapter at a time. Callers
        // that pile up behind this lock are almost always racing toward the same
        // desk (boot + StrictMode remount, off->on recovery), so once one wins
        // the rest just observe the connection and return.
        let _guard = self.connect_lock.lock().await;
        if self.is_connected().await {
            return true;
        }

        // the streaming scan (if any) owns the adapter; release it before we
        // re-scan for the target address inside find_peripheral.
        self.scan_stop().await;

        self.shared
            .connection(ConnectionState::Connecting, name, Some(address));

        let fail = || {
            if emit_failure {
                self.shared
                    .connection(ConnectionState::Disconnected, None, None);
            }
        };

        let (peripheral, discovered_name) = match self.find_peripheral(address).await {
            Ok(Some(found)) => found,
            _ => {
                fail();
                return false;
            }
        };
        // LINAK desks gate their characteristics behind an authenticated link, so
        // bond before connecting; without this the first read fails with
        // INSUFFICIENT_AUTHENTICATION. No-op off Windows. See [`pairing`].
        if pairing::ensure_paired(address).await.is_err() {
            fail();
            return false;
        }
        if peripheral.connect().await.is_err() {
            fail();
            return false;
        }
        if self.setup_connection(address, &peripheral).await.is_err() {
            fail();
            let _ = peripheral.disconnect().await;
            return false;
        }

        let connected_name = discovered_name.unwrap_or_else(|| address.to_string());
        self.shared.connection(
            ConnectionState::Connected,
            Some(&connected_name),
            Some(address),
        );
        true
    }

    async fn setup_connection(&self, address: &str, peripheral: &Peripheral) -> Result<()> {
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
            address: address.to_string(),
            peripheral: peripheral.clone(),
            move_c,
            refout_c,
            refin_c,
        });
        Ok(())
    }

    /// Disconnect, unsubscribe, and clear the cached height. Leaves the Windows
    /// bond intact, so an internal teardown (Bluetooth toggled off, app exit)
    /// doesn't force a re-pair on the next connect. An explicit user disconnect
    /// goes through [`disconnect_and_unpair`](Self::disconnect_and_unpair).
    pub async fn disconnect(&self) {
        let conn = self.conn.lock().await.take();
        if let Some(conn) = conn {
            if conn.peripheral.is_connected().await.unwrap_or(false) {
                let _ = conn.peripheral.unsubscribe(&conn.refout_c).await;
                let _ = conn.peripheral.disconnect().await;
            }
        }
        *self.shared.height.lock().unwrap() = None;
        self.shared
            .connection(ConnectionState::Disconnected, None, None);
    }

    /// Disconnect and remove the Windows bond, fully releasing the desk so
    /// another device (the phone app, another PC) can connect. Dropping the BLE
    /// link alone leaves the desk bonded to this PC, so from the user's side it
    /// is still taken; this is what the "Disconnect desk" action calls. No-op
    /// beyond [`disconnect`](Self::disconnect) off Windows.
    pub async fn disconnect_and_unpair(&self) {
        let address = self.conn.lock().await.as_ref().map(|c| c.address.clone());
        self.disconnect().await;
        if let Some(address) = address {
            let _ = pairing::unpair(&address).await;
        }
    }
}
