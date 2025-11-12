#!/bin/bash
set -e

REPO="unhappychoice/gitlogue"
BINARY_NAME="gitlogue"

get_latest_release() {
    curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
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
