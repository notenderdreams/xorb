#!/usr/bin/env bash
set -e

REPO="notenderdreams/xorb"
BIN_NAME="xorb"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    if [ "$ARCH" = "x86_64" ]; then
      ASSET="xorb-linux-x86_64.tar.gz"
    else
      echo "Unsupported architecture for Linux: $ARCH"
      exit 1
    fi
    ;;
  darwin)
    if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
      ASSET="xorb-macos-aarch64.tar.gz"
    elif [ "$ARCH" = "x86_64" ]; then
      ASSET="xorb-macos-x86_64.tar.gz"
    else
      echo "Unsupported architecture for macOS: $ARCH"
      exit 1
    fi
    ;;
  *)
    echo "Unsupported operating system: $OS. For Windows, please run install.ps1 in PowerShell."
    exit 1
    ;;
esac

DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
INSTALL_DIR="${HOME}/.local/bin"
TARGET="${INSTALL_DIR}/${BIN_NAME}"

mkdir -p "$INSTALL_DIR"

TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

echo "Downloading ${BIN_NAME} for ${OS}/${ARCH}..."
curl -fsSL "$DOWNLOAD_URL" -o "${TEMP_DIR}/${ASSET}"

echo "Extracting binary..."
tar -xzf "${TEMP_DIR}/${ASSET}" -C "$TEMP_DIR"

rm -f "$TARGET"
mv -f "${TEMP_DIR}/${BIN_NAME}" "$TARGET"
chmod +x "$TARGET"

echo ""
echo "✓ ${BIN_NAME} successfully installed/updated to ${TARGET}"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo ""
  echo "Notice: ${INSTALL_DIR} is not in your current PATH."
  echo "Add the following line to your ~/.bashrc or ~/.zshrc:"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
echo "Run '${BIN_NAME} --help' to get started!"
