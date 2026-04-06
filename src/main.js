// SPDX-License-Identifier: MIT
'use strict';

const COLORS = [
  '#000000', '#ff0000', '#00ff00', '#0000ff',
  '#ff00ff', '#00ffff', '#ffff00', '#ffffff',
];

function hexToRgb(hex) {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 0xFF, (n >> 8) & 0xFF, n & 0xFF];
}

function updateStatus(devices) {
  const el = document.getElementById('status');
  if (devices.length > 0) {
    el.textContent = `blink(1) device found: ${devices[0]}`;
    el.className = 'found';
  } else {
    el.textContent = 'No blink(1) devices found';
    el.className = '';
  }
}

async function init() {
  // Build swatch buttons
  const grid = document.getElementById('swatches');
  for (const color of COLORS) {
    const btn = document.createElement('button');
    btn.className = 'swatch';
    btn.style.background = color;
    btn.title = color;
    btn.addEventListener('click', () => {
      const [r, g, b] = hexToRgb(color);
      window.blink1.setColor(r, g, b);
    });
    grid.appendChild(btn);
  }

  // Rescan button
  document.getElementById('rescan-btn').addEventListener('click', async () => {
    const result = await window.blink1.rescan();
    updateStatus(result.devices);
  });

  // Initial device check
  const devices = await window.blink1.getDevices();
  updateStatus(Array.isArray(devices) ? devices : []);
}

document.addEventListener('DOMContentLoaded', init);
