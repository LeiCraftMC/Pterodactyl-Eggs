#!/bin/bash

CYAN="\033[38;2;0;255;255m"
RED="\033[38;2;255;0;0m"
BOLD="\033[1m"
NC='\033[0m'


# Define the root directory to /home/container.
# We can only write in /home/container and /tmp in the container.
ROOTFS_DIR=/home/container

export PATH=$PATH:~/.local/usr/bin


function install() {
    if [ ! -e "$ROOTFS_DIR/.installed" ]; then
            if [ "$EXPERIMENTAL" -ne 1 ]; then
            # Download run.sh
            curl -Ls -H 'Pragma: no-cache' \
                    -H 'Cache-Control: no-cache, no-store' \
                    "https://raw.githubusercontent.com/LeiCraft/Pterodactyl-Eggs/main/Debain11/install.sh" -o "$ROOTFS_DIR/install.sh"
            # Make run.sh executable.
            chmod +x "$ROOTFS_DIR/install.sh"
        fi

        bash "$ROOTFS_DIR/install.sh"
    fi
}


function downloadPostInstallationFiles() {
    if [ "$EXPERIMENTAL" -ne 1 ] || [ ! -e "$ROOTFS_DIR/home/container/postInstallation.sh" ]; then
        # Download postInstallation.sh
        curl -Ls -H 'Pragma: no-cache' \
                -H 'Cache-Control: no-cache, no-store' \
                "https://raw.githubusercontent.com/LeiCraft/Pterodactyl-Eggs/main/Debain11/postInstallation.sh" -o "$ROOTFS_DIR/home/container/postInstallation.sh"
        # Make postInstallation.sh executable.
        chmod +x "$ROOTFS_DIR/home/container/postInstallation.sh"
    fi

    if [ "$EXPERIMENTAL" -ne 1 ] || [ ! -e "$ROOTFS_DIR/home/container/preLoginAll.sh" ]; then
        # Download preLoginAll.sh
        curl -Ls -H 'Pragma: no-cache' \
                -H 'Cache-Control: no-cache, no-store' \
                "https://raw.githubusercontent.com/LeiCraft/Pterodactyl-Eggs/main/Debain11/preLoginAll.sh" -o "$ROOTFS_DIR/home/container/preLoginAll.sh"
        # Make preLoginAll.sh executable.
        chmod +x "$ROOTFS_DIR/home/container/preLoginAll.sh"
    fi

    if [ "$EXPERIMENTAL" -ne 1 ] || [ ! -e "$ROOTFS_DIR/home/container/preLoginUser.sh" ]; then
        # Download preLoginUser.sh
        curl -Ls -H 'Pragma: no-cache' \
                -H 'Cache-Control: no-cache, no-store' \
                "https://raw.githubusercontent.com/LeiCraft/Pterodactyl-Eggs/main/Debain11/preLoginUser.sh" -o "$ROOTFS_DIR/home/container/preLoginUser.sh"
        # Make preLoginUser.sh executable.
        chmod +x "$ROOTFS_DIR/home/container/preLoginUser.sh"
    fi
}


function start() {

    if [ ! -e "/.postInstallationMade" ]; then
        downloadPostInstallationFiles
    fi

    if [ "$EXPERIMENTAL" -ne 1 ] && [ ! -e "$ROOTFS_DIR/home/container/run.sh" ]; then
        # Download run.sh
        curl -Ls -H 'Pragma: no-cache' \
                -H 'Cache-Control: no-cache, no-store' \
                "https://raw.githubusercontent.com/LeiCraft/Pterodactyl-Eggs/main/Debain11/run.sh" -o "$ROOTFS_DIR/home/container/run.sh"
        # Make run.sh executable.
        chmod +x "$ROOTFS_DIR/home/container/run.sh"
    fi

    ###########################
    # Start PRoot environment #
    ###########################

    # Get all ports from vps.config
    port_args=""
    while read line; do
        case "$line" in
            internalip=*) ;;
            port[0-9]*=*) port=${line#*=}; if [ -n "$port" ]; then port_args=" -p $port:$port$port_args"; fi;;
            port=*) port=${line#*=}; if [ -n "$port" ]; then port_args=" -p $port:$port$port_args"; fi;;   
        esac
    done < "$ROOTFS_DIR/vps.config"

    # This command starts PRoot and binds several important directories
    # from the host file system to our special root file system.
    "$ROOTFS_DIR/usr/local/bin/proot" \
    --rootfs="${ROOTFS_DIR}" \
    -0 -w "/root" -b /dev -b /sys -b /proc -b /etc/resolv.conf $port_args --kill-on-exit \
    /bin/bash "$ROOTFS_DIR/run.sh"

}

install
start
