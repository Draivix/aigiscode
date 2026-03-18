#!/usr/bin/env bash
# AigisCode Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
#    or: wget -qO- https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash

set -euo pipefail

VERSION="${AIGISCODE_VERSION:-0.1.0}"
REPO="Draivix/aigiscode"
BIN_NAME="aigiscode"

if [ -z "${NO_COLOR:-}" ] && [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    PURPLE='\033[0;35m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' PURPLE='' BOLD='' RESET=''
fi

info()  { printf "${BLUE}▸${RESET} %s\n" "$*"; }
ok()    { printf "${GREEN}✓${RESET} %s\n" "$*"; }
warn()  { printf "${YELLOW}!${RESET} %s\n" "$*"; }
err()   { printf "${RED}✗${RESET} %s\n" "$*" >&2; }
die()   { err "$*"; exit 1; }

banner() {
    printf "\n"
    printf "${PURPLE}${BOLD}"
    printf "    ╔═══════════════════════════════════════╗\n"
    printf "    ║     🛡️  AigisCode Installer  v%s   ║\n" "$VERSION"
    printf "    ║     Native Rust CLI                  ║\n"
    printf "    ╚═══════════════════════════════════════╝\n"
    printf "${RESET}\n"
}

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        CYGWIN*|MINGW*|MSYS*) echo "windows" ;;
        *)       die "Unsupported operating system: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x64" ;;
        arm64|aarch64) echo "arm64" ;;
        *)             echo "$(uname -m)" ;;
    esac
}

find_cargo() {
    if command -v cargo >/dev/null 2>&1; then
        echo "cargo"
        return 0
    fi
    if [ -x "$HOME/.cargo/bin/cargo" ]; then
        echo "$HOME/.cargo/bin/cargo"
        return 0
    fi
    return 1
}

install_rustup() {
    info "Rust toolchain not found. Installing rustup..."
    command -v curl >/dev/null 2>&1 || die "curl is required to install Rust"
    curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal || \
        die "Failed to install rustup"
}

install_with_cargo() {
    local cargo_cmd="$1"
    info "Installing native Rust CLI with cargo..."
    "$cargo_cmd" install \
        --git "https://github.com/${REPO}.git" \
        --locked \
        aigiscore \
        --bin "$BIN_NAME" 2>/dev/null || \
    "$cargo_cmd" install \
        --git "https://github.com/${REPO}.git" \
        aigiscore \
        --bin "$BIN_NAME" || \
        die "Failed to install ${BIN_NAME} with cargo"
}

ensure_path() {
    local target_dir="$HOME/.cargo/bin"
    if [[ ":$PATH:" != *":$target_dir:"* ]]; then
        warn "$target_dir is not in your PATH"

        local shell_name
        shell_name=$(basename "${SHELL:-/bin/bash}")
        local rc_file=""

        case "$shell_name" in
            zsh)  rc_file="$HOME/.zshrc" ;;
            bash) rc_file="$HOME/.bashrc" ;;
            fish) rc_file="$HOME/.config/fish/config.fish" ;;
        esac

        if [ -n "$rc_file" ]; then
            local path_line='export PATH="$HOME/.cargo/bin:$PATH"'
            if [ "$shell_name" = "fish" ]; then
                path_line='set -gx PATH $HOME/.cargo/bin $PATH'
            fi

            if [ -f "$rc_file" ] && ! grep -q '.cargo/bin' "$rc_file" 2>/dev/null; then
                echo "" >> "$rc_file"
                echo "# Added by AigisCode installer" >> "$rc_file"
                echo "$path_line" >> "$rc_file"
                ok "Added $target_dir to PATH in $rc_file"
                info "Run: source $rc_file  (or restart your terminal)"
            fi
        fi
    fi
}

verify_install() {
    local aigis_cmd=""
    if command -v "$BIN_NAME" >/dev/null 2>&1; then
        aigis_cmd="$BIN_NAME"
    elif [ -x "$HOME/.cargo/bin/$BIN_NAME" ]; then
        aigis_cmd="$HOME/.cargo/bin/$BIN_NAME"
    fi

    if [ -n "$aigis_cmd" ]; then
        ok "AigisCode installed successfully! ($("$aigis_cmd" --version 2>/dev/null || echo unknown))"
    else
        ok "AigisCode installed. You may need to restart your terminal."
    fi
}

main() {
    banner

    local os arch cargo_cmd
    os=$(detect_os)
    arch=$(detect_arch)
    info "Detected: ${os}/${arch}"

    if ! cargo_cmd=$(find_cargo); then
        install_rustup
        cargo_cmd=$(find_cargo || true)
    fi
    [ -n "${cargo_cmd:-}" ] || die "Cargo is required but was not found after rustup installation."

    ok "Found $("$cargo_cmd" --version 2>&1)"
    install_with_cargo "$cargo_cmd"
    ensure_path
    verify_install

    printf "\n"
    printf "${BOLD}Get started:${RESET}\n"
    printf "  ${GREEN}aigiscode analyze .${RESET}    # Analyze current directory and write native artifacts\n"
    printf "  ${GREEN}aigiscode surface .${RESET}    # Generate architecture-surface.json\n"
    printf "  ${GREEN}aigiscode --help${RESET}       # See all commands\n"
    printf "\n"
    printf "  ${BLUE}Docs:${RESET}    https://aigiscode.com/docs\n"
    printf "  ${BLUE}GitHub:${RESET}  https://github.com/${REPO}\n"
    printf "\n"
}

main "$@"
