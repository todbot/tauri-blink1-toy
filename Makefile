.PHONY: dev build dist-mac dist-mac-unsigned dist-win dist-win-unsigned icons

# Run in development mode (hot-reload frontend, Rust backend rebuilt on change)
dev:
	cargo tauri dev

# Build release binary (no installer)
build:
	cargo tauri build

# macOS signed + notarized universal DMG (arm64 + x64)
# Requires: APPLE_ID, APPLE_APP_SPECIFIC_PASSWORD, APPLE_TEAM_ID env vars
dist-mac:
	cargo tauri build --target universal-apple-darwin

# macOS unsigned build for local testing
dist-mac-unsigned:
	cargo tauri build --target universal-apple-darwin \
	  --config '{"bundle":{"macOS":{"signingIdentity":null}}}'

# Windows NSIS installer, signed via Azure Trusted Signing
# Requires: AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET
# Install signing tool first: cargo install trusted-signing-cli
dist-win:
	cargo tauri build --target x86_64-pc-windows-msvc

# Windows unsigned build for local testing
dist-win-unsigned:
	cargo tauri build --target x86_64-pc-windows-msvc \
	  --config '{"bundle":{"windows":{"signCommand":null}}}'

# Generate all required icon sizes from a 1024x1024 source PNG
# Usage: make icons SRC=/path/to/icon-1024.png
icons:
	cargo tauri icon $(SRC)

# Extract 1024px PNG from the Electron project's icns for use as icon source
extract-icon:
	sips -s format png ../../../node/electron-blink1-toy/pkg/icon.icns \
	     --out /tmp/blink1-toy-icon-1024.png \
	     --resampleHeightWidth 1024 1024
	$(MAKE) icons SRC=/tmp/blink1-toy-icon-1024.png
