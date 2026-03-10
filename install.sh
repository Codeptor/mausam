#!/bin/sh
set -e

REPO="codeptor/mausam"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
            aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
            *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)  TARGET="x86_64-apple-darwin" ;;
            arm64)   TARGET="aarch64-apple-darwin" ;;
            *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS (use Windows builds from GitHub Releases)"
        exit 1
        ;;
esac

# Get latest version tag
VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')"
if [ -z "$VERSION" ]; then
    echo "Failed to fetch latest release"
    exit 1
fi

ARCHIVE="mausam-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

echo "Installing mausam ${VERSION} (${TARGET})..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" -o "${TMPDIR}/${ARCHIVE}"
tar xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

if [ -w "$INSTALL_DIR" ]; then
    mv "${TMPDIR}/mausam" "${INSTALL_DIR}/mausam"
else
    sudo mv "${TMPDIR}/mausam" "${INSTALL_DIR}/mausam"
fi

chmod +x "${INSTALL_DIR}/mausam"

echo "mausam ${VERSION} installed to ${INSTALL_DIR}/mausam"
