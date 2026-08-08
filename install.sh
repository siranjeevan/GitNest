#!/usr/bin/env bash
# GitNest One-Line Installer Script for macOS and Linux

set -e

REPO="siranjeevan/GitNest"
VERSION="1.0.0"

echo "◈ Installing GitNest v${VERSION}..."

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "${OS}" in
  darwin)
    if [ "${ARCH}" = "arm64" ]; then
      ARTIFACT="gitnest-v${VERSION}-macos-arm64.tar.gz"
    else
      ARTIFACT="gitnest-v${VERSION}-macos-x86_64.tar.gz"
    fi
    ;;
  linux)
    if [ "${ARCH}" = "x86_64" ]; then
      ARTIFACT="gitnest-v${VERSION}-linux-x86_64.tar.gz"
    else
      echo "Error: Unsupported architecture ${ARCH} on Linux."
      exit 1
    fi
    ;;
  *)
    echo "Error: Unsupported operating system ${OS}."
    exit 1
    ;;
esac

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARTIFACT}"
TMP_DIR="$(mktemp -d)"

echo "Downloading ${DOWNLOAD_URL}..."
curl -sSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/${ARTIFACT}"

echo "Extracting..."
tar -xzf "${TMP_DIR}/${ARTIFACT}" -C "${TMP_DIR}"

INSTALL_DIR="/usr/local/bin"
if [ ! -w "${INSTALL_DIR}" ]; then
  INSTALL_DIR="${HOME}/.local/bin"
  mkdir -p "${INSTALL_DIR}"
fi

echo "Installing binary to ${INSTALL_DIR}/gitnest..."
mv "${TMP_DIR}/gitnest" "${INSTALL_DIR}/gitnest"
chmod +x "${INSTALL_DIR}/gitnest"
rm -rf "${TMP_DIR}"

echo ""
echo "✓ GitNest v${VERSION} installed successfully!"
echo "Run 'gitnest' to start the interactive dashboard."
