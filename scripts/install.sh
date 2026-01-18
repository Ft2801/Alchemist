#!/bin/bash
# Alchemist Installer Script
# Usage: curl -fsSL https://alchemist.sh/install | bash

set -e

REPO="yourusername/alchemist"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="alchemist"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_logo() {
    echo -e "${MAGENTA}"
    echo '     _    _      _                    _     _   '
    echo '    / \  | | ___| |__   ___ _ __ ___ (_)___| |_ '
    echo '   / _ \ | |/ __| '_ \ / _ \ '_ ` _ \| / __| __|'
    echo '  / ___ \| | (__| | | |  __/ | | | | | \__ \ |_ '
    echo ' /_/   \_\_|\___|_| |_|\___|_| |_| |_|_|___/\__|'
    echo -e "${NC}"
    echo -e "${CYAN}    ⚗️  Transform JSON into Type-Safe Code  ⚗️${NC}"
    echo ""
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Detect OS and Architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)     OS="linux" ;;
        Darwin*)    OS="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *)          error "Unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH="x86_64" ;;
        arm64|aarch64)  ARCH="aarch64" ;;
        *)              error "Unsupported architecture: $ARCH" ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    info "Detected platform: $PLATFORM"
}

# Get latest release version
get_latest_version() {
    info "Fetching latest version..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v?([^"]+)".*/\1/')
    
    if [ -z "$VERSION" ]; then
        error "Failed to get latest version"
    fi
    
    success "Latest version: v$VERSION"
}

# Download and install
install() {
    local DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/alchemist-${PLATFORM}.tar.gz"
    local TMP_DIR=$(mktemp -d)
    
    info "Downloading from: $DOWNLOAD_URL"
    
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/alchemist.tar.gz"; then
        error "Failed to download binary"
    fi
    
    info "Extracting..."
    tar -xzf "$TMP_DIR/alchemist.tar.gz" -C "$TMP_DIR"
    
    info "Installing to $INSTALL_DIR..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TMP_DIR/alchemist" "$INSTALL_DIR/$BINARY_NAME"
    else
        warn "Need sudo to install to $INSTALL_DIR"
        sudo mv "$TMP_DIR/alchemist" "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Cleanup
    rm -rf "$TMP_DIR"
    
    success "Alchemist installed successfully!"
}

verify() {
    if command -v alchemist &> /dev/null; then
        echo ""
        success "Installation verified!"
        echo ""
        alchemist --version
        echo ""
        echo -e "${GREEN}🎉 You're all set!${NC}"
        echo ""
        echo -e "Get started with:"
        echo -e "  ${CYAN}alchemist -i data.json -t typescript${NC}"
        echo ""
    else
        warn "alchemist not found in PATH"
        echo "You may need to add $INSTALL_DIR to your PATH"
    fi
}

main() {
    print_logo
    detect_platform
    get_latest_version
    install
    verify
}

main
