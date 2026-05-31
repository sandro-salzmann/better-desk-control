//! Bonding a desk before we talk to it.
//!
//! LINAK desks expose their control characteristics behind an authenticated
//! link: reading or writing them over an unbonded connection fails with
//! `E_BLUETOOTH_ATT_INSUFFICIENT_AUTHENTICATION` (HRESULT `0x80650005`).
//!
//! On Windows the implicit pairing that WinRT kicks off on that first failed
//! read - the "add a device" popup - uses a ceremony the desk rejects, so it
//! never bonds and every connect bounces back to the scan screen. We instead
//! drive an explicit `ConfirmOnly` pairing, which is the "just works" ceremony
//! the desk actually supports, and auto-accept it. Windows persists the bond,
//! so later connects authenticate silently.
//!
//! Off Windows this is a no-op: CoreBluetooth (macOS) and BlueZ (Linux) bond
//! transparently on first authenticated access.

use crate::Result;

/// Ensure the desk at `address` is bonded, pairing it if not. A no-op if it is
/// already paired (or on a platform that pairs transparently).
///
/// Every WinRT interface handle is `!Send`, so none may be held across an await
/// in the connect future (Tauri spawns it, which requires `Send`). The pairing
/// therefore runs entirely synchronously on a blocking thread; this `async`
/// wrapper only awaits the join handle, which is `Send`.
#[cfg(target_os = "windows")]
pub(super) async fn ensure_paired(address: &str) -> Result<()> {
    let address = address.to_string();
    tokio::task::spawn_blocking(move || pair_blocking(&address))
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            format!("pairing task failed: {e}").into()
        })?
}

#[cfg(target_os = "windows")]
fn pair_blocking(address: &str) -> Result<()> {
    use windows::Devices::Bluetooth::BluetoothLEDevice;
    use windows::Devices::Enumeration::{
        DeviceInformationCustomPairing, DevicePairingKinds, DevicePairingProtectionLevel,
        DevicePairingRequestedEventArgs, DevicePairingResultStatus,
    };
    use windows::Foundation::TypedEventHandler;

    // `IAsyncOperation::get` blocks via an event-signalled completion handler
    // (no STA message pump), so it is safe on a `spawn_blocking` thread.
    let device = BluetoothLEDevice::FromBluetoothAddressAsync(parse_address(address)?)?.get()?;
    let pairing = device.DeviceInformation()?.Pairing()?;
    if pairing.IsPaired()? {
        return Ok(());
    }

    let custom = pairing.Custom()?;
    // "Just works" pairing: accept the moment the desk asks to confirm.
    let handler = TypedEventHandler::<
        DeviceInformationCustomPairing,
        DevicePairingRequestedEventArgs,
    >::new(|_sender, args| {
        if let Some(args) = args.as_ref() {
            args.Accept()?;
        }
        Ok(())
    });
    let token = custom.PairingRequested(&handler)?;
    let result = custom
        .PairWithProtectionLevelAsync(
            DevicePairingKinds::ConfirmOnly,
            DevicePairingProtectionLevel::Encryption,
        )?
        .get();
    // Always detach the handler, even if pairing failed.
    custom.RemovePairingRequested(token)?;

    match result?.Status()? {
        DevicePairingResultStatus::Paired | DevicePairingResultStatus::AlreadyPaired => Ok(()),
        status => Err(format!("pairing failed: {status:?}").into()),
    }
}

#[cfg(not(target_os = "windows"))]
pub(super) async fn ensure_paired(_address: &str) -> Result<()> {
    Ok(())
}

/// Parse a colon-delimited MAC (`DF:EA:BA:E8:8E:44`) into the big-endian `u64`
/// that [`BluetoothLEDevice::FromBluetoothAddressAsync`] expects.
#[cfg(target_os = "windows")]
fn parse_address(address: &str) -> Result<u64> {
    let mut octets = 0u64;
    let mut count = 0;
    for part in address.split(':') {
        let octet = u8::from_str_radix(part, 16)
            .map_err(|_| format!("invalid Bluetooth address: {address}"))?;
        octets = (octets << 8) | u64::from(octet);
        count += 1;
    }
    if count != 6 {
        return Err(format!("invalid Bluetooth address: {address}").into());
    }
    Ok(octets)
}
