#!/usr/bin/env bash
set -euo pipefail

echo "=========================================================="
echo "  ReplicantWave - Full Multi-Format Packaging Engine     "
echo "  Formats: VST3, CLAP, AU, Standalone                    "
echo "=========================================================="

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${PROJECT_DIR}"

mkdir -p dist

echo "==> [1/4] Building and Bundling VST3 & CLAP plugins..."
cargo xtask bundle replicantwave --release

echo "==> [2/4] Building Standalone binary..."
cargo build --release --bin replicantwave-standalone

echo "==> [3/4] Packaging AudioUnit format..."
bash tools/build_au.sh || true

echo "==> [4/4] Assembling release distribution..."
DIST_DIR="${PROJECT_DIR}/dist/ReplicantWave-v1.0.1"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

# Copy CLAP
if [ -e target/bundled/replicantwave.clap ]; then
    cp -r target/bundled/replicantwave.clap "${DIST_DIR}/"
fi

# Copy VST3
if [ -d target/bundled/replicantwave.vst3 ]; then
    cp -r target/bundled/replicantwave.vst3 "${DIST_DIR}/"
fi

# Copy Standalone
if [ -f target/release/replicantwave-standalone ]; then
    cp target/release/replicantwave-standalone "${DIST_DIR}/"
elif [ -f target/release/replicantwave-standalone.exe ]; then
    cp target/release/replicantwave-standalone.exe "${DIST_DIR}/"
fi

# Copy AU if present
if [ -d target/bundled/ReplicantWave.component ]; then
    cp -r target/bundled/ReplicantWave.component "${DIST_DIR}/"
fi

# Copy License and Readme
cp LICENSE "${DIST_DIR}/"
cp README.md "${DIST_DIR}/"

echo "Distribution package ready at ${DIST_DIR}"
