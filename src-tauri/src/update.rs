//! Self-update: check on launch, download in the background, and hold the
//! verified bytes until the user clicks "Restart now".
//!
//! Rust owns the whole flow (check -> download -> install), matching the
//! "Rust owns control-flow decisions" convention. The frontend only renders the
//! `update-*` window events and calls `update_install` from the restart button:
//!
//! * `update-available` ({ version }) once a newer release is found,
//! * `update-progress`  ({ downloaded, total }) per chunk while downloading,
//! * `update-ready`     ({ version }) once the verified bytes are in hand.
//!
//! Nothing is emitted when the app is up to date, offline, or the check fails,
//! so a normal launch stays silent.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

/// A downloaded, signature-verified update waiting to be installed.
struct ReadyUpdate {
    update: Update,
    bytes: Vec<u8>,
}

/// Managed state: the pending update, set once the background download finishes.
#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<ReadyUpdate>>);

#[derive(Clone, Serialize)]
struct VersionEvent {
    version: String,
}

#[derive(Clone, Serialize)]
struct ProgressEvent {
    downloaded: usize,
    total: Option<u64>,
}

/// Check for an update and, if one exists, download it in the background. The
/// verified bytes are stashed in [`PendingUpdate`] for [`update_install`] to
/// apply. Spawned from `setup`; it stays quiet on the no-update / offline /
/// failed paths so launches without an update show nothing.
pub async fn check_and_download(app: AppHandle) {
    let Ok(updater) = app.updater() else {
        return;
    };
    // No update, an unreachable endpoint, or a malformed manifest: stay silent.
    let Ok(Some(update)) = updater.check().await else {
        return;
    };

    let _ = app.emit(
        "update-available",
        VersionEvent {
            version: update.version.clone(),
        },
    );

    let mut downloaded = 0usize;
    let mut last_emit = 0usize;
    let app_progress = app.clone();
    let bytes = update
        .download(
            move |chunk, total| {
                downloaded += chunk;
                // Throttle: a multi-MB download arrives in many small chunks, so
                // emit at ~1% steps (or every 256 KB when the server sent no
                // length) instead of an IPC message per chunk. The bar is
                // replaced by `update-ready` on completion, so a missed final
                // sub-step doesn't matter.
                let step = total.map_or(256 * 1024, |t| (t / 100).max(1) as usize);
                if downloaded - last_emit >= step {
                    last_emit = downloaded;
                    let _ =
                        app_progress.emit("update-progress", ProgressEvent { downloaded, total });
                }
            },
            || {},
        )
        .await;
    let Ok(bytes) = bytes else {
        return; // download or signature verification failed
    };

    let version = update.version.clone();
    *app.state::<PendingUpdate>().0.lock().unwrap() = Some(ReadyUpdate { update, bytes });
    let _ = app.emit("update-ready", VersionEvent { version });
}

/// Install the downloaded update and relaunch into it. Wired to the "Restart
/// now" button. On Windows `install` exits the app itself (installer
/// limitation), so the `restart` below only runs on the other platforms.
#[tauri::command]
pub fn update_install(app: AppHandle, pending: State<'_, PendingUpdate>) -> Result<(), String> {
    let Some(ready) = pending.0.lock().unwrap().take() else {
        return Err("no update is ready to install".into());
    };
    ready
        .update
        .install(ready.bytes)
        .map_err(|e| e.to_string())?;
    app.restart()
}
