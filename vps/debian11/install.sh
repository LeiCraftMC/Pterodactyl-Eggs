#!/bin/bash

CYAN="\033[38;2;0;255;255m"
RED="\033[38;2;255;0;0m"
GREEN="\033[38;2;0;255;0m"
BOLD="\033[1m"
NC='\033[0m'

# Define the root directory to /home/container.
# We can only write in /home/container and /tmp in the container.
ROOTFS_DIR=/home/container

PROOT_VERSION="5.3.0" # Some releases do not have static builds attached.

# Detect the machine architecture.
ARCH=$(uname -m)

#############################
# Linux Installation #
#############################

function installOS() {

    # Download & decompress the Linux root file system

    printf "\033c"
    printf "${CYAN}╭───────────────────────────────────────────────────────────────────────────────╮${NC}\n"
    printf "${CYAN}│                                                                               │${NC}\n"
    printf "${CYAN}│                              ${BOLD}LeiCraft_MC Hosting${NC}${CYAN}                              │${NC}\n"
    printf "${CYAN}│                                                                               │${NC}\n"
    printf "${CYAN}│                                   ${BOLD}${RED}Debain 11${NC}${CYAN}                                   │${NC}\n"
    printf "${CYAN}│                                                                               │${NC}\n"
    printf "${CYAN}╰───────────────────────────────────────────────────────────────────────────────╯${NC}\n\n"
  
    printf "${GREEN}Installing Debian 11...${NC}\n"
    url="https://fra1lxdmirror01.do.letsbuildthe.cloud/images/debian/bullseye/amd64/default/"

    LATEST_VERSION=$(curl -s $url | grep -oP 'href="\K[^"]+/' | sort -r | head -n 1)

    curl -Ls "${url}${LATEST_VERSION}/rootfs.tar.xz" -o $ROOTFS_DIR/rootfs.tar.xz
    tar -xf $ROOTFS_DIR/rootfs.tar.xz -C "$ROOTFS_DIR"
    mkdir $ROOTFS_DIR/home/container/ -p

}

################################
# Package Installation & Setup #
#################################

function installProot() {
    # Download static proot.
    # Download the packages from their sources
    mkdir -p "$ROOTFS_DIR/usr/local/bin"
    curl -Ls "https://github.com/proot-me/proot/releases/download/v${PROOT_VERSION}/proot-v${PROOT_VERSION}-${ARCH}-static" -o "$ROOTFS_DIR/usr/local/bin/proot"
    # Make PRoot executable.
    chmod 755 "$ROOTFS_DIR/usr/local/bin/proot"
}


function addDNSResolver() {
    # Add DNS Resolver nameservers to resolv.conf.
    printf "nameserver 1.1.1.1\nnameserver 1.0.0.1" > "${ROOTFS_DIR}/etc/resolv.conf"
}

function afterInstallationCleanup() {
    # Clean-up after installation complete & finish up.
    # Wipe the files we downloaded into /tmp previously.
    rm -rf $ROOTFS_DIR/rootfs.tar.xz /tmp/sbin
    # Create .installed to later check whether OS is installed.
    touch "$ROOTFS_DIR/.installed"
}


installOS
installProot
addDNSResolver
afterInstallationCleanup