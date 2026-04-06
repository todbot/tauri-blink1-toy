// SPDX-License-Identifier: MIT
//
// Tauri commands — port of Electron main.js IPC handlers for blink(1).

mod blink1;

use blink1::Blink1;
use hidapi::HidApi;
use serde_json::{json, Value};
use std::sync::Mutex;

// ── Shared state ─────────────────────────────────────────────────────────────

pub struct AppState {
    api: HidApi,
    b1: Blink1,
}

type SharedState = Mutex<AppState>;

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Set blink(1) color. Auto-opens device if not already open.
/// Mirrors Electron `blink1:setColor` handler.
#[tauri::command]
fn blink1_set_color(r: u8, g: u8, b: u8, state: tauri::State<SharedState>) -> Value {
    let mut st = state.lock().unwrap();
    if !st.b1.is_open() {
        // Refresh device list before attempting open.
        // Split borrows explicitly: api and b1 are separate fields.
        let _ = st.api.refresh_devices();
        let AppState { ref api, ref mut b1 } = *st;
        if let Err(e) = b1.open(api) {
            return json!({"ok": false, "error": e});
        }
    }
    match st.b1.fade_to_rgb(100, r, g, b) {
        Ok(_) => json!({"ok": true}),
        Err(e) => {
            // Device may have been unplugged; close so next call rescans
            st.b1.close();
            json!({"ok": false, "error": e})
        }
    }
}

/// Rescan for blink(1) devices, reconnect if found.
/// Mirrors Electron `blink1:rescan` handler.
#[tauri::command]
fn blink1_rescan(state: tauri::State<SharedState>) -> Value {
    let mut st = state.lock().unwrap();
    st.b1.close();
    let _ = st.api.refresh_devices();
    let devices = Blink1::list_devices(&st.api);
    if !devices.is_empty() {
        let AppState { ref api, ref mut b1 } = *st;
        let _ = b1.open(api);
    }
    json!({"ok": true, "devices": devices})
}

/// List currently visible blink(1) devices without reconnecting.
/// Mirrors Electron `blink1:getDevices` handler.
#[tauri::command]
fn blink1_get_devices(state: tauri::State<SharedState>) -> Value {
    let mut st = state.lock().unwrap();
    let _ = st.api.refresh_devices();
    let devices = Blink1::list_devices(&st.api);
    json!(devices)  // returns a JSON array of serial strings
}

// ── App entry point ───────────────────────────────────────────────────────────

pub fn run() {
    let api = HidApi::new().expect("Failed to initialize HidApi");
    let state: SharedState = Mutex::new(AppState {
        api,
        b1: Blink1::new(),
    });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            blink1_set_color,
            blink1_rescan,
            blink1_get_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
