//! Discovery, connect/disconnect, and characteristic setup.

use std::time::Duration;

use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use tokio::time::sleep;
use uuid::Uuid;

use super::{Conn, DeskController};
use crate::protocol::{CHAR_DPG, CHAR_MOVE, CHAR_REFIN, CHAR_REFOUT, DESK_NAME_PREFIX};
use crate::reporter::DeskInfo;
use crate::Result;

impl DeskController {
    async fn central(&self) -> Result<Adapter> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        adapters
            .into_iter()
            .next()
            .ok_or_else(|| "no Bluetooth adapter found".into())
    }

    /// Discover nearby desks. Does not require a connection.
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
                        });
                    }
                }
            }
        }
        Ok(out)
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
        self.emit_status(format!("connecting to {address}…"));
        let peripheral = match self.find_peripheral(address).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                self.emit_status("device not found");
                return false;
            }
            Err(e) => {
                self.emit_status(format!("scan failed: {e}"));
                return false;
            }
        };
        if let Err(e) = peripheral.connect().await {
            self.emit_status(format!("connect failed: {e}"));
            return false;
        }
        if let Err(e) = self.setup_connection(&peripheral).await {
            self.emit_status(format!("setup failed: {e}"));
            let _ = peripheral.disconnect().await;
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
        let move_c = find(CHAR_MOVE)?;
        let refout_c = find(CHAR_REFOUT)?;
        let dpg_c = find(CHAR_DPG)?;
        let refin_c = find(CHAR_REFIN)?;

        peripheral.subscribe(&refout_c).await?;
        peripheral.subscribe(&dpg_c).await?;

        // prime the height reading
        if let Ok(data) = peripheral.read(&refout_c).await {
            self.shared.on_height(&data);
        }

        // background task that fans incoming notifications to the shared state
        let mut stream = peripheral.notifications().await?;
        let shared = self.shared.clone();
        tokio::spawn(async move {
            while let Some(n) = stream.next().await {
                if n.uuid == CHAR_REFOUT {
                    shared.on_height(&n.value);
                } else if n.uuid == CHAR_DPG {
                    shared.on_dpg(&n.value);
                }
            }
        });

        *self.conn.lock().await = Some(Conn {
            peripheral: peripheral.clone(),
            move_c,
            refout_c,
            dpg_c,
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
                let _ = conn.peripheral.unsubscribe(&conn.dpg_c).await;
                let _ = conn.peripheral.disconnect().await;
            }
        }
        *self.shared.height.lock().unwrap() = None;
        self.emit_status("disconnected");
    }
}
