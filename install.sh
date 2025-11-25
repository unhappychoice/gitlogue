#!/bin/bash
set -e

REPO="unhappychoice/gitlogue"
BINARY_NAME="gitlogue"

get_latest_release() {
    curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

check_glibc_version() {
    if ! command -v ldd &> /dev/null; then
        return
    fi

    GLIBC_VERSION=$(ldd --version 2>&1 | head -n1 | grep -oE '[0-9]+\.[0-9]+' | head -1)
    REQUIRED_VERSION="2.39"

    if [ -n "$GLIBC_VERSION" ]; then
        if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$GLIBC_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
            echo ""
            echo "WARNING: Your glibc version ($GLIBC_VERSION) is older than $REQUIRED_VERSION."
            echo "The pre-built binary may not work on your system."
            echo ""
            echo "Options:"
            echo "  1. Upgrade your OS to get a newer glibc"
            echo "  2. Build from source: cargo install gitlogue"
            echo "  3. Continue anyway and see if it works"
            echo ""
            read -p "Continue with installation? [y/N] " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                echo "Installation cancelled."
                exit 0
            fi
        fi
    fi
}

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            check_glibc_version
            case "$ARCH" in
                x86_64)
                    echo "x86_64-unknown-linux-gnu"
                    ;;
                aarch64|arm64)
                    echo "aarch64-unknown-linux-gnu"
                    ;;
                *)
                    echo "Unsupported architecture: $ARCH" >&2
                    exit 1
                    ;;
            esac
            ;;
        Darwin*)
            case "$ARCH" in
                x86_64)
                    echo "x86_64-apple-darwin"
                    ;;
                arm64)
                    echo "aarch64-apple-darwin"
                    ;;
                *)
                    echo "Unsupported architecture: $ARCH" >&2
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $OS" >&2
            exit 1
            ;;
    esac
}

main() {
    VERSION="${1:-$(get_latest_release)}"
    PLATFORM="$(detect_platform)"

    if [ -z "$VERSION" ]; then
        echo "Failed to detect latest version" >&2
        exit 1
    fi

    echo "Installing $BINARY_NAME $VERSION for $PLATFORM..."

    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/${BINARY_NAME}-${VERSION}-${PLATFORM}.tar.gz"
    TEMP_DIR="$(mktemp -d)"

    trap 'rm -rf "$TEMP_DIR"' EXIT

    echo "Downloading from $DOWNLOAD_URL..."
    curl -sL "$DOWNLOAD_URL" | tar xz -C "$TEMP_DIR"

    INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
    mkdir -p "$INSTALL_DIR"

    mv "$TEMP_DIR/$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    echo "Successfully installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"
    echo ""
    echo "Make sure $INSTALL_DIR is in your PATH:"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
}

main "$@"
