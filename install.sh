#!/usr/bin/env sh
# mdnotes installer for Linux and macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/rellit/mdnotes/main/install.sh | sh
set -e

REPO="rellit/mdnotes"
INSTALL_DIR="$HOME/.local/bin"

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64) ARCHIVE="mdnotes-linux-x86_64.tar.gz" ;;
            *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64) ARCHIVE="mdnotes-macos-x86_64.tar.gz" ;;
            arm64)  ARCHIVE="mdnotes-macos-aarch64.tar.gz" ;;
            *)      echo "Unsupported architecture: $ARCH" && exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

LATEST_URL="https://github.com/$REPO/releases/latest/download/$ARCHIVE"

mkdir -p "$INSTALL_DIR"

TMP_DIR=$(mktemp -d)
# shellcheck disable=SC2064
trap "rm -rf '$TMP_DIR'" EXIT

echo "Downloading $ARCHIVE..."
curl -fsSL "$LATEST_URL" -o "$TMP_DIR/$ARCHIVE"

echo "Extracting..."
tar -xzf "$TMP_DIR/$ARCHIVE" -C "$TMP_DIR"

for bin in mdn mdnui; do
    if [ -f "$TMP_DIR/$bin" ]; then
        cp "$TMP_DIR/$bin" "$INSTALL_DIR/$bin"
        chmod +x "$INSTALL_DIR/$bin"
        echo "Installed $bin -> $INSTALL_DIR/$bin"
    fi
done

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "NOTE: $INSTALL_DIR is not in your PATH."
        echo "Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        ;;
esac

echo ""
echo "mdnotes installed successfully!"
echo "Run 'mdn --help' to get started."
