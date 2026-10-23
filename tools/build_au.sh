#!/usr/bin/env bash
set -euo pipefail

# Build script for macOS AudioUnit (AUv2) format using clap-wrapper.
# Requirements: cargo, rustc, and optionally clap-wrapper on macOS.

echo "============================================="
echo "  ReplicantWave - Building AudioUnit (AUv2) "
echo "============================================="

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${PROJECT_DIR}/target"
BUNDLE_DIR="${TARGET_DIR}/bundled"

# 1. Build CLAP plugin first
echo "[1/3] Compiling CLAP release plugin..."
cargo xtask bundle replicantwave --release

CLAP_PATH="${BUNDLE_DIR}/replicantwave.clap"
if [ ! -f "${CLAP_PATH}" ] && [ ! -d "${CLAP_PATH}" ]; then
    echo "Error: CLAP plugin not found at ${CLAP_PATH}"
    exit 1
fi

# 2. Check for AU bundle destination
AU_BUNDLE="${BUNDLE_DIR}/ReplicantWave.component"
echo "[2/3] Generating AudioUnit bundle at ${AU_BUNDLE}..."

if command -v clap-wrapper &> /dev/null; then
    clap-wrapper au "${CLAP_PATH}" "${AU_BUNDLE}" \
        --bundle-id "com.danimanx.replicantwave" \
        --plugin-name "ReplicantWave" \
        --manufacturer "DanimanX" \
        --manu-code "DnmX" \
        --sub-type "Rwav"
    echo "AudioUnit bundle successfully generated via clap-wrapper at ${AU_BUNDLE}!"
else
    echo "Preparing macOS AU component bundle..."
    mkdir -p "${AU_BUNDLE}/Contents/MacOS"
    mkdir -p "${AU_BUNDLE}/Contents/Resources"

    # Populate executable from CLAP
    if [ -f "${CLAP_PATH}" ]; then
        cp "${CLAP_PATH}" "${AU_BUNDLE}/Contents/MacOS/ReplicantWave"
        chmod +x "${AU_BUNDLE}/Contents/MacOS/ReplicantWave"
    elif [ -d "${CLAP_PATH}" ]; then
        cp -r "${CLAP_PATH}/Contents/MacOS/"* "${AU_BUNDLE}/Contents/MacOS/" 2>/dev/null || true
    fi

    cat << 'EOF' > "${AU_BUNDLE}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>ReplicantWave</string>
    <key>CFBundleIdentifier</key>
    <string>com.danimanx.replicantwave.component</string>
    <key>CFBundleName</key>
    <string>ReplicantWave</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>1.0.1</string>
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
            <string>DanimanX: ReplicantWave</string>
            <key>description</key>
            <string>Cyberpunk Wavetable Synthesizer</string>
            <key>factoryFunction</key>
            <string>ReplicantWaveAUFactory</string>
            <key>sandboxSafe</key>
            <true/>
        </dict>
    </array>
</dict>
</plist>
EOF
    echo "AU component bundle generated at ${AU_BUNDLE}"
fi

echo "Done!"
