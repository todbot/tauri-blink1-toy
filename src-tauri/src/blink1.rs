// SPDX-License-Identifier: MIT
//
// Rust driver for blink(1) USB HID device.
// Protocol: 9-byte HID feature reports (report ID 0x01).
// Reference: https://github.com/todbot/blink1/blob/main/docs/blink1-hid-commands.md

use hidapi::HidApi;

pub const BLINK1_VID: u16 = 0x27B8;
pub const BLINK1_PID: u16 = 0x01ED;

const REPORT_ID: u8 = 0x01;

pub struct Blink1 {
    device: Option<hidapi::HidDevice>,
}

impl Blink1 {
    pub fn new() -> Self {
        Self { device: None }
    }

    pub fn is_open(&self) -> bool {
        self.device.is_some()
    }

    /// Returns serial numbers of all connected blink(1) devices.
    pub fn list_devices(api: &HidApi) -> Vec<String> {
        api.device_list()
            .filter(|d| d.vendor_id() == BLINK1_VID && d.product_id() == BLINK1_PID)
            .map(|d| d.serial_number().unwrap_or("blink(1)").to_string())
            .collect()
    }

    /// Opens the first available blink(1). Returns Err if none found or open fails.
    pub fn open(&mut self, api: &HidApi) -> Result<(), String> {
        if self.device.is_some() {
            return Ok(());
        }
        let device = api
            .open(BLINK1_VID, BLINK1_PID)
            .map_err(|e| e.to_string())?;
        self.device = Some(device);
        Ok(())
    }

    pub fn close(&mut self) {
        self.device = None;
    }

    /// Fade to an RGB color over `fade_ms` milliseconds.
    /// cmd 'c' (0x63): [report_id, 'c', r, g, b, th, tl, 0, 0]
    /// fade time units are 10ms; th:tl is big-endian.
    pub fn fade_to_rgb(&self, fade_ms: u16, r: u8, g: u8, b: u8) -> Result<(), String> {
        let dev = self.device.as_ref().ok_or("blink(1) not open")?;
        let ticks = fade_ms / 10;
        let th = (ticks >> 8) as u8;
        let tl = (ticks & 0xFF) as u8;
        let report = [REPORT_ID, 0x63, r, g, b, th, tl, 0, 0];
        dev.send_feature_report(&report)
            .map_err(|e| format!("blink1 write error: {e}"))?;
        Ok(())
    }
}
