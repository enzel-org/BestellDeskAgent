#!/usr/bin/env bash
set -euo pipefail

REPO="enzel-org/BestellDeskAgent"
BIN_NAME="BestellDeskAgent"
INSTALL_PATH="/usr/local/bin/${BIN_NAME}"
SERVICE_NAME="bestelldesk-agent"
ENV_DIR="/etc/${SERVICE_NAME}"
ENV_FILE="${ENV_DIR}/agent.env"
SYSTEMD_UNIT="/etc/systemd/system/${SERVICE_NAME}.service"

# --- helpers ---
have_cmd() { command -v "$1" >/dev/null 2>&1; }

fetch() {
    local url="$1"
    if have_cmd curl; then
        curl -sSL "$url"
    elif have_cmd wget; then
        wget -qO- "$url"
    else
        echo "Error: neither curl nor wget found" >&2
        exit 1
    fi
}

download_file() {
    local url="$1" dest="$2"
    if have_cmd curl; then
        curl -L --fail -o "$dest" "$url"
    elif have_cmd wget; then
        wget -O "$dest" "$url"
    else
        echo "Error: neither curl nor wget found" >&2
        exit 1
    fi
}

get_latest_release() {
    fetch "https://api.github.com/repos/${REPO}/releases/latest" \
      | grep "tag_name" | cut -d '"' -f4
}

download_binary() {
    local version="$1"
    local url="https://github.com/${REPO}/releases/download/${version}/${BIN_NAME}-linux-x86_64"
    echo "Downloading ${url} ..."
    download_file "${url}" "${INSTALL_PATH}"
    chmod +x "${INSTALL_PATH}"
    echo "Installed to ${INSTALL_PATH}"
}

create_service() {
    sudo useradd -r -s /usr/sbin/nologin ${SERVICE_NAME} 2>/dev/null || true
    sudo install -d -o ${SERVICE_NAME} -g ${SERVICE_NAME} "${ENV_DIR}"

    if [ ! -f "${ENV_FILE}" ]; then
        cat <<EOF | sudo tee "${ENV_FILE}" >/dev/null
MONGODB_URI="mongodb+srv://user:pass@cluster0.example.net/mydb"
AGENT_BIND="0.0.0.0:8443"
EOF
        sudo chmod 600 "${ENV_FILE}"
        echo "Created default ${ENV_FILE}, edit it with: sudo $0 edit"
    fi

    cat <<EOF | sudo tee "${SYSTEMD_UNIT}" >/dev/null
[Unit]
Description=BestellDesk Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${SERVICE_NAME}
Group=${SERVICE_NAME}
ExecStart=${INSTALL_PATH}
EnvironmentFile=${ENV_FILE}

NoNewPrivileges=true
ProtectSystem=full
ProtectHome=true
PrivateTmp=true
ProtectHostname=true
ProtectClock=true
ProtectKernelModules=true
ProtectKernelTunables=true
ProtectControlGroups=true
LockPersonality=true

Restart=on-failure
RestartSec=2s

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable --now ${SERVICE_NAME}
    echo "Service installed and started."
}

uninstall() {
    echo "Stopping and disabling service..."
    sudo systemctl stop ${SERVICE_NAME} || true
    sudo systemctl disable ${SERVICE_NAME} || true
    sudo rm -f "${SYSTEMD_UNIT}"
    sudo systemctl daemon-reload

    echo "Removing binary and config..."
    sudo rm -f "${INSTALL_PATH}"
    sudo rm -rf "${ENV_DIR}"

    echo "Removing user..."
    sudo userdel ${SERVICE_NAME} 2>/dev/null || true

    echo "Uninstalled."
}

edit_env() {
    ${EDITOR:-nano} "${ENV_FILE}"
    echo "Reloading service..."
    sudo systemctl daemon-reload
    sudo systemctl restart ${SERVICE_NAME}
}

usage() {
    echo "Usage: $0 {install|uninstall|edit} [version]"
    echo
    echo "Or with VERSION env:"
    echo "  VERSION=v0.1.0 $0 install"
    echo
    echo "Examples:"
    echo "  $0 install         # install latest"
    echo "  $0 install v0.1.0  # install specific version"
    echo "  $0 edit            # edit /etc/${SERVICE_NAME}/agent.env"
    echo "  $0 uninstall       # remove everything"
}

main() {
    local action="${1:-}"
    case "${action}" in
        install)
            local version="${2:-${VERSION:-}}"
            if [ -z "$version" ]; then
                version=$(get_latest_release)
            fi
            echo "Installing BestellDeskAgent ${version}..."
            download_binary "${version}"
            create_service
            ;;
        uninstall)
            uninstall
            ;;
        edit)
            edit_env
            ;;
        *)
            usage
            ;;
    esac
}

main "$@"
