#!/usr/bin/env bash

set -euo pipefail

sdk_root="${ANDROID_SDK_ROOT:-/opt/android-sdk}"
tools_version="15859902"
tools_archive="commandlinetools-linux-${tools_version}_latest.zip"
tools_checksum="4e4c464f145a7512b57d088ac6c278c03c9eea610886b35a5e0804e74eedf583"
download_dir="$(mktemp -d)"

cleanup() {
    rm -rf "${download_dir}"
}
trap cleanup EXIT

mkdir -p "${sdk_root}/cmdline-tools"
curl --fail --location --retry 3 \
    "https://dl.google.com/android/repository/${tools_archive}" \
    --output "${download_dir}/${tools_archive}"
echo "${tools_checksum}  ${download_dir}/${tools_archive}" | sha256sum --check
unzip -q "${download_dir}/${tools_archive}" -d "${download_dir}/tools"
rm -rf "${sdk_root}/cmdline-tools/latest"
mv "${download_dir}/tools/cmdline-tools" "${sdk_root}/cmdline-tools/latest"

"${sdk_root}/cmdline-tools/latest/bin/android" \
    --no-metrics \
    --sdk="${sdk_root}" \
    sdk install \
    "platform-tools" \
    "platforms/android-34" \
    "build-tools/34.0.0" \
    "ndk/27.2.12479018"
