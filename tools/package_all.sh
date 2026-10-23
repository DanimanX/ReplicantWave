#!/usr/bin/env bash
set -euo pipefail

echo "=========================================================="
echo "  ReplicatWave - Full Multi-Format Packaging Engine      "
echo "  Formats: VST3, CLAP, AU, Standalone                    "
echo "=========================================================="

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${PROJECT_DIR}"

mkdir -p dist

echo "==> [1/4] Building and Bundling VST3 & CLAP plugins..."
cargo xtask bundle replicatwave --release

echo "==> [2/4] Building Standalone binary..."
cargo build --release --bin replicatwave-standalone

echo "==> [3/4] Packaging AudioUnit format..."
bash tools/build_au.sh || true

echo "==> [4/4] Assembling release distribution..."
DIST_DIR="${PROJECT_DIR}/dist/ReplicatWave-v1.0.0"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

# Copy CLAP
if [ -f target/bundled/replicatwave.clap ]; then
    cp -r target/bundled/replicatwave.clap "${DIST_DIR}/"
fi

# Copy VST3
if [ -d target/bundled/replicatwave.vst3 ]; then
    cp -r target/bundled/replicatwave.vst3 "${DIST_DIR}/"
fi

# Copy Standalone
if [ -f target/release/replicatwave-standalone ]; then
    cp target/release/replicatwave-standalone "${DIST_DIR}/"
fi

# Copy AU if present
if [ -d target/bundled/ReplicatWave.component ]; then
    cp -r target/bundled/ReplicatWave.component "${DIST_DIR}/"
fi

# Copy License and Readme
cp LICENSE "${DIST_DIR}/"
cp README.md "${DIST_DIR}/"

echo "Release package created at: ${DIST_DIR}"
ls -la "${DIST_DIR}"
