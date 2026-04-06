# tauri-blink1-toy

A simple [Tauri v2](https://tauri.app) app for controlling a [blink(1)](https://blink1.thingm.com) USB LED device.

Click a color swatch to fade the blink(1) to that color. Use the Rescan button to detect a device that was plugged in after launch.

This is a port of [electron-blink1-toy](../node/electron-blink1-toy/) — same UI and behavior, rewritten with a Rust/Tauri backend instead of Electron/Node. USB HID access uses the [`hidapi`](https://crates.io/crates/hidapi) crate directly, with no native Node modules required.

## Requirements

- [Rust](https://rustup.rs) (stable)
- [Tauri CLI](https://tauri.app/start/): `cargo install tauri-cli`
- A [blink(1)](https://blink1.thingm.com) USB device (mk1, mk2, or mk3)
- macOS, Windows, or Linux

## Usage

```
make dev
```

The app scans for a blink(1) on launch. If none is found, plug one in and click **Rescan**.

## Build

```
make build              # release binary for current platform
make dist-mac-unsigned  # unsigned universal macOS .app (arm64 + x86_64)
make dist-mac           # signed + notarized universal macOS DMG (requires env vars below)
make dist-win           # Windows NSIS installer
```

### macOS code signing & notarization

```bash
export APPLE_ID="your@email.com"
export APPLE_APP_SPECIFIC_PASSWORD="xxxx-xxxx-xxxx-xxxx"
export APPLE_TEAM_ID="XXXXXXXXXX"
make dist-mac
```

`APPLE_APP_SPECIFIC_PASSWORD` must be an [App-Specific Password](https://support.apple.com/en-us/102431), not your Apple ID password.

### Icons

To regenerate icons from the original Electron project's `icon.icns`:

```
make extract-icon
```

Or supply your own 1024×1024 PNG:

```
make icons SRC=/path/to/icon-1024.png
```

## Architecture

| Layer | Technology |
|---|---|
| UI | Vanilla HTML/CSS/JS (`src/`) |
| IPC bridge | `src/blink1-bridge.js` — wraps `window.__TAURI__.core.invoke` |
| Backend | Rust (`src-tauri/src/lib.rs`) — three Tauri commands |
| USB/HID | `src-tauri/src/blink1.rs` — `hidapi` crate, VID `0x27B8` / PID `0x01ED` |

The three Tauri commands map 1:1 to the original Electron IPC channels:

| Tauri command | Electron channel |
|---|---|
| `blink1_set_color(r, g, b)` | `blink1:setColor` |
| `blink1_rescan()` | `blink1:rescan` |
| `blink1_get_devices()` | `blink1:getDevices` |

## See also

- [`electron-blink1-toy`](../node/electron-blink1-toy/) — the original Electron version
- [`BlinkMSequencerTauri`](../BlinkMSequencerNew/BlinkMSequencerTauri/) — a more complete Tauri app using the same patterns, targeting the BlinkM/LinkM device
