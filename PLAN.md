# Plan: Port electron-blink1-toy → Tauri

## Context

`electron-blink1-toy` is a simple Electron + React app that controls a blink(1) USB LED device (set color, get devices, rescan). The goal is to rewrite it as a Tauri v2 app using the patterns and structure established in `BlinkMSequencerTauri` (same author, same USB HID approach, production-quality reference). The result should be lighter, native, and distribution-ready.

---

## Reference & Source

| | Path |
|---|---|
| Source (Electron) | `~/projects/node/electron-blink1-toy/` |
| Reference (Tauri) | `~/projects/BlinkMSequencerNew/BlinkMSequencerTauri/` |
| Output | `~/projects/tauri/tauri-blink1-toy/` |

---

## Architecture Mapping

| Electron concept | Tauri equivalent |
|---|---|
| `electron/main.js` IPC handlers | Rust `#[tauri::command]` fns in `lib.rs` |
| `node-blink1` / `node-hid` | `hidapi` Rust crate (same as BlinkMSequencer) |
| `electron/preload.js` contextBridge | `src/blink1-bridge.js` (mirrors `linkm-bridge.js`) |
| `window.electronAPI` | `window.blink1` |
| React + Bootstrap UI | Vanilla HTML/CSS/JS (mirrors BlinkMSequencer pattern) |

---

## Project Structure

```
tauri-blink1-toy/
├── Makefile                      # dev / build / dist targets (copy from BlinkMSequencer)
├── src/                          # Frontend (static, no build step)
│   ├── index.html
│   ├── main.js                   # UI logic (color clicks, rescan, status)
│   ├── blink1-bridge.js          # Tauri invoke wrapper (mirrors linkm-bridge.js)
│   └── style.css
└── src-tauri/
    ├── Cargo.toml                # hidapi, serde, tauri v2
    ├── tauri.conf.json           # 500×500 window, dist-dir ../src
    ├── build.rs
    ├── entitlements.plist        # com.apple.security.device.usb = true
    ├── capabilities/
    │   └── default.json          # core:default + event permissions
    └── src/
        ├── main.rs               # calls lib::run()
        ├── lib.rs                # Tauri commands + AppState
        └── blink1.rs             # HID driver (VID 0x27B8, PID 0x01ED)
```

---

## Step 1 — Scaffold

```
cd ~/projects/tauri
mkdir tauri-blink1-toy && cd tauri-blink1-toy
mkdir -p src src-tauri/src src-tauri/capabilities src-tauri/icons
```

---

## Step 2 — Rust: `src-tauri/Cargo.toml`

Dependencies (mirror BlinkMSequencer, minus tokio — no async needed):
```toml
tauri = { version = "2", features = ["devtools"] }
tauri-build = { version = "2" }
hidapi = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## Step 3 — Rust: `src-tauri/src/blink1.rs`

HID driver for blink(1). Device identifiers:
- **VID:** `0x27B8`  **PID:** `0x01ED`

Protocol: 9-byte HID feature reports.
```
fadeToRGB(fade_ms, r, g, b):
  report = [0x01, 0x63, r, g, b, (fade_ms/10 >> 8) as u8, (fade_ms/10 & 0xFF) as u8, 0, 0]
  hid_device.send_feature_report(&report)
```

Functions:
- `Blink1::list_devices(api: &HidApi) -> Vec<String>` — serial numbers of connected devices
- `Blink1::open(&mut self, api: &HidApi) -> Result<()>` — open first device
- `Blink1::fade_to_rgb(&self, fade_ms: u16, r: u8, g: u8, b: u8) -> Result<()>`
- `Blink1::close(&mut self)` — drop handle

Reference: `BlinkMSequencerTauri/src-tauri/src/linkm.rs` for HidApi init and feature-report send pattern.

---

## Step 4 — Rust: `src-tauri/src/lib.rs`

**AppState:**
```rust
pub struct AppState {
    api: HidApi,
    b1: Blink1,
}
```
Wrapped in `Mutex<AppState>`, managed by Tauri.

**Tauri commands** — 1:1 mapping to Electron IPC channels:

| Command | Electron channel | Returns |
|---|---|---|
| `blink1_set_color(r,g,b)` | `blink1:setColor` | `{ok: bool, error?: String}` |
| `blink1_rescan()` | `blink1:rescan` | `{ok: bool, devices: Vec<String>}` |
| `blink1_get_devices()` | `blink1:getDevices` | `Vec<String>` |

- `blink1_set_color`: auto-opens device if not connected; closes on write error so next call rescans.
- `blink1_rescan`: close + `api.refresh_devices()` + reopen; return device list.
- `blink1_get_devices`: `api.refresh_devices()` + enumerate; no reconnect.

**Borrow-checker note:** `b1.open(&api)` requires splitting borrows explicitly since both fields are behind the same `MutexGuard`:
```rust
let AppState { ref api, ref mut b1 } = *st;
b1.open(api)?;
```

---

## Step 5 — Tauri Config: `tauri.conf.json`

```json
{
  "productName": "blink1-toy",
  "version": "0.1.0",
  "identifier": "com.todbot.blink1-toy",
  "build": { "frontendDist": "../src" },
  "app": {
    "windows": [{ "width": 500, "height": 500, "resizable": false, "center": true }],
    "withGlobalTauri": true
  }
}
```

`withGlobalTauri: true` is required so `blink1-bridge.js` can use `window.__TAURI__.core.invoke` without a build step.

---

## Step 6 — macOS: `entitlements.plist`

Copy from BlinkMSequencer verbatim — `com.apple.security.device.usb = true` is required for `hidapi` under the hardened runtime.

---

## Step 7 — Frontend: `src/blink1-bridge.js`

Mirror of `linkm-bridge.js`. Exposes `window.blink1`:
```javascript
const { invoke } = window.__TAURI__.core;

window.blink1 = {
  setColor:   (r, g, b) => invoke('blink1_set_color', { r, g, b }),
  rescan:     ()        => invoke('blink1_rescan'),
  getDevices: ()        => invoke('blink1_get_devices'),
};
```

---

## Step 8 — Frontend: `src/index.html` + `src/main.js`

Replace React/Bootstrap with Vanilla JS (no build step, matches BlinkMSequencer pattern).

UI from `Blink1Toy.jsx`:
- Title: "blink(1) Toy"
- `<p id="status">` — device connection status
- 8 color swatch `<button>` elements (black, red, green, blue, magenta, cyan, yellow, white)
- Rescan button

Key logic in `main.js`:
```javascript
// On load
const devices = await window.blink1.getDevices()
updateStatus(devices)

// Swatch click
window.blink1.setColor(r, g, b)

// Rescan
const { devices } = await window.blink1.rescan()
updateStatus(devices)
```

Color conversion: inline `hexToRgb()` — no `tinycolor2` needed for simple hex colors.

---

## Step 9 — Makefile

Based on BlinkMSequencer's Makefile:
- `make dev` → `cargo tauri dev`
- `make build` → `cargo tauri build`
- `make dist-mac-unsigned` → universal macOS build for local testing
- `make dist-mac` → signed + notarized universal DMG
- `make extract-icon` → pulls icon from electron app's icns, generates all sizes

---

## Icons

Placeholder: copy from BlinkMSequencerTauri during development.
Final: `make extract-icon` pulls from `electron-blink1-toy/pkg/icon.icns` and regenerates all sizes via `cargo tauri icon`.

---

## Verification

1. `make dev` — window opens, no compile errors
2. Plug in blink(1) → device serial appears in status line
3. Click a color swatch → LED changes color with 100ms fade
4. Unplug → Rescan → status shows "No blink(1) devices found"
5. Plug back in → Rescan → device found, colors work again
6. `make dist-mac-unsigned` → `.app` bundle launches and controls device
