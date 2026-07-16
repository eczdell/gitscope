#!/usr/bin/env bash
set -euo pipefail

REPO="eczdell/gitscope"
BIN_NAME="gitscope"
INSTALL_DIR="/usr/local/bin"

# ─── Colors ────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log()  { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}==>${NC} $1"; }
err()  { echo -e "${RED}==>${NC} $1"; }

# ─── Detect platform ───────────────────────────────────────────────────────
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

case "$ARCH" in
    x86_64|amd64)  ARCH="x86_64"  ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)
        err "Unsupported architecture: $ARCH"
        err "Please build from source: cargo install --git https://github.com/$REPO"
        exit 1
        ;;
esac

case "$OS" in
    linux)   OS="unknown-linux-gnu"   ;;
    darwin)  OS="apple-darwin"         ;;
    mingw*|msys*|cygwin*)
        err "Windows detected. Please use one of these methods instead:"
        err "  cargo install ${BIN_NAME}"
        err "  or download the .exe from: https://github.com/$REPO/releases"
        exit 1
        ;;
    *)
        err "Unsupported OS: $OS"
        exit 1
        ;;
esac

TARGET="${ARCH}-${OS}"

# ─── Determine install prefix ──────────────────────────────────────────────
# Allow override via INSTALL_DIR env var
if [ -n "${INSTALL_DIR:-}" ]; then
    :  # already set
elif [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    # Fall back to ~/.local/bin or ~/bin
    if [ -d "$HOME/.local/bin" ] && [ -w "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        INSTALL_DIR="$HOME/bin"
    fi
    warn "No write access to /usr/local/bin, installing to $INSTALL_DIR"
fi

# Ensure install dir exists
mkdir -p "$INSTALL_DIR"

# ─── Fetch latest release ──────────────────────────────────────────────────
log "Fetching latest release info for ${REPO}..."

LATEST_URL="https://api.github.com/repos/${REPO}/releases/latest"
RELEASE_JSON=$(curl -fsSL "$LATEST_URL" 2>/dev/null || true)

if [ -z "$RELEASE_JSON" ]; then
    warn "No release found. Falling back to building from source..."
    if command -v cargo &>/dev/null; then
        log "Installing via cargo..."
        cargo install --git "https://github.com/${REPO}.git" "$BIN_NAME"
        log "Done! ${BIN_NAME} is now available at $(which ${BIN_NAME})"
        exit 0
    else
        err "cargo not found. Install Rust first: https://rustup.rs"
        err "Then run: cargo install --git https://github.com/${REPO}.git ${BIN_NAME}"
        exit 1
    fi
fi

# ─── Find the right asset ──────────────────────────────────────────────────
ASSET_URL=$(echo "$RELEASE_JSON" | \
    python3 -c "
import json, sys
data = json.load(sys.stdin)
for asset in data.get('assets', []):
    name = asset['name']
    if '$TARGET' in name or '$ARCH-$OS' in name:
        print(asset['browser_download_url'])
        break
" 2>/dev/null || true)

if [ -z "$ASSET_URL" ]; then
    warn "No pre-built binary found for ${TARGET}. Building from source..."
    if command -v cargo &>/dev/null; then
        log "Installing via cargo..."
        cargo install --git "https://github.com/${REPO}.git" "$BIN_NAME"
        log "Done! ${BIN_NAME} is now available at $(which ${BIN_NAME})"
        exit 0
    else
        err "cargo not found. Install Rust first: https://rustup.rs"
        exit 1
    fi
fi

# ─── Download and install ──────────────────────────────────────────────────
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

log "Downloading ${BIN_NAME} for ${TARGET}..."
curl -fsSL "$ASSET_URL" -o "$TMPDIR/${BIN_NAME}.tar.gz"

log "Extracting..."
tar -xzf "$TMPDIR/${BIN_NAME}.tar.gz" -C "$TMPDIR"

log "Installing to ${INSTALL_DIR}/${BIN_NAME}..."
install -m 755 "$TMPDIR/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

# ─── Verify ────────────────────────────────────────────────────────────────
if command -v "${INSTALL_DIR}/${BIN_NAME}" &>/dev/null; then
    log "Installation complete!"
    "${INSTALL_DIR}/${BIN_NAME}" --version
else
    warn "Binary installed but not in PATH."
    warn "Add ${INSTALL_DIR} to your PATH or move the binary manually."
fi

echo ""
log "Run '${BIN_NAME}' to start the TUI."
log "Or '${BIN_NAME} --help' for all options."

