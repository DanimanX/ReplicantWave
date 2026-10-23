#!/usr/bin/env bash
set -euo pipefail

# Build script for macOS AudioUnit (AUv2) format using clap-wrapper.
# Requirements: cargo, rustc, and optionally clap-wrapper on macOS.

echo "============================================="
echo "  ReplicatWave - Building AudioUnit (AUv2)  "
echo "============================================="

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${PROJECT_DIR}/target"
BUNDLE_DIR="${TARGET_DIR}/bundled"

# 1. Build CLAP plugin first
echo "[1/3] Compiling CLAP release plugin..."
cargo xtask bundle replicatwave --release

CLAP_PATH="${BUNDLE_DIR}/replicatwave.clap"
if [ ! -f "${CLAP_PATH}" ] && [ ! -d "${CLAP_PATH}" ]; then
    echo "Error: CLAP plugin not found at ${CLAP_PATH}"
    exit 1
fi

# 2. Check for clap-wrapper tool
echo "[2/3] Checking for AU wrapper tools..."
# On macOS or systems with clap-wrapper / cmake AU tools

# 3. Wrap CLAP to AU
AU_BUNDLE="${BUNDLE_DIR}/ReplicatWave.component"
echo "[3/3] Generating AudioUnit bundle at ${AU_BUNDLE}..."

if command -v clap-wrapper &> /dev/null; then
    clap-wrapper au "${CLAP_PATH}" "${AU_BUNDLE}" \
        --bundle-id "com.danimanx.replicatwave" \
        --plugin-name "ReplicatWave" \
        --manufacturer "DanimanX" \
        --manu-code "DnmX" \
        --sub-type "Rwav"
    echo "AudioUnit bundle successfully generated at ${AU_BUNDLE}!"
else
    echo "clap-wrapper not available in this environment. Preparing AU bundle skeleton..."
    mkdir -p "${AU_BUNDLE}/Contents/MacOS"
    mkdir -p "${AU_BUNDLE}/Contents/Resources"
    cat << 'EOF' > "${AU_BUNDLE}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>ReplicatWave</string>
    <key>CFBundleIdentifier</key>
    <string>com.danimanx.replicatwave.component</string>
    <key>CFBundleName</key>
    <string>ReplicatWave</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>AudioComponents</key>
    <array>
        <dict>
            <key>type</key>
            <string>aumu</string>
            <key>subtype</key>
            <string>Rwav</string>
            <key>manufacturer</key>
            <string>DnmX</string>
            <key>name</key>
            <string>DanimanX: ReplicatWave</string>
            <key>description</key>
            <string>Cyberpunk Wavetable Synthesizer</string>
            <key>factoryFunction</key>
            <string>ReplicatWaveAUFactory</string>
            <key>sandboxSafe</key>
            <true/>
        </dict>
    </array>
</dict>
</plist>
EOF
    echo "Skeleton created. To build binary AU component on macOS, run: clap-wrapper au target/bundled/replicatwave.clap target/bundled/ReplicatWave.component"
fi

echo "Done!"
