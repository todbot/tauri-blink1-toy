// SPDX-License-Identifier: MIT
//
// Tauri bridge — exposes window.blink1.* API using Tauri invoke,
// replacing Electron's preload.js / contextBridge.
'use strict';

const { invoke } = window.__TAURI__.core;

window.blink1 = {
  setColor:   (r, g, b) => invoke('blink1_set_color', { r, g, b }),
  rescan:     ()        => invoke('blink1_rescan'),
  getDevices: ()        => invoke('blink1_get_devices'),
};
