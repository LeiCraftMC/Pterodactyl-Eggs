#!/bin/bash

function check_cpu_arch {
    local arch=$(uname -m)
    if [ "$arch" = "x86_64" ]; then
        declare -g ARCH="linux_amd64"
    elif [ "$arch" = "aarch64" ]; then
        declare -g ARCH="linux_arm64"
    else
        echo "Unsupported architecture: $arch"
        exit 1
    fi
}

function get_latest_version {
    local api_url="https://api.github.com/repos/go-gost/gost/releases"
    local releases_json=$(curl -s "$api_url")
    echo "$releases_json" | jq -r '[.[] | select(.prerelease == false)][0].tag_name'
}

function get_current_version {
    local version=$(./gost -V 2>/dev/null)
    echo "$version" | sed -n 's/^gost v\([0-9]\+\.[0-9]\+\.[0-9]\+\).*/\1/p'
}

function download_ntfy {
    local version=$1
    local arch=$2
    local name="gost_$(echo $version | cut -c2-)_${arch}"
    local url="https://github.com/go-gost/gost/releases/download/${version}/${name}.tar.gz"
    echo $url

    echo "Downloading gost version $version for architecture $arch..."

    http_response_code="$(curl --write-out '%{http_code}' -sL -o $name.tar.gz "$url")"

    if [ "$http_response_code" != "200" ]; then
        echo "Failed to download gost binary. HTTP response code: $http_response_code"
        exit 1
    fi

    echo "Download complete. Extracting..."

    mkdir -p $name
    tar -xzf "${name}.tar.gz" -C ./$name
    if [ $? -ne 0 ]; then
        echo "Failed to extract gost binary."
        exit 1
    fi
    
    cp -a ${name}/gost /home/container/gost
    chmod +x /home/container/gost

    rm "${name}.tar.gz"
    rm -rf $name

    echo "gost $version downloaded successfully."
}

function create_default_config {
    mkdir -p /home/container/conf

    if [ ! -f /home/container/conf/gost.yml ]; then
        echo "Creating default config file..."
        cat <<EOF > /home/container/conf/gost.yml
services:

- name: socks5-proxy
  addr: ":$SERVER_PORT"
  handler:
    type: socks
    auther: auther-main
  listener:
    type: tcp

authers:
- name: auther-main
  file:
    path: /home/container/conf/accouns

chains:

- name: tor-backend
  hops:
  - name: hop-0
    nodes:
    - name: tor-backend-0
      addr: 127.0.0.1:9151
      connector:
        type: socks5
      dialer:
        type: tcp

EOF
    fi

    if [ ! -f /home/container/conf/accounts ]; then
        echo "Creating default config file..."
        cat <<EOF > /home/container/conf/accounts
# username password

admin changeme
EOF
    fi
}

function main {

    echo "Starting..."

    cd /home/container

    check_cpu_arch

    if [ -z "$VERSION" ]; then
        echo "No VERSION specified. Using latest version..."
        export VERSION="latest"
    fi

    # Extract Startup CMD
    if [ -n "$STARTUP" ]; then
        STARTUP_CMD=$(echo ${STARTUP} | sed -e 's/{{/${/g' -e 's/}}/}/g')
    else
        echo "No STARTUP_CMD specified. Running fallback command..."
        export STARTUP_CMD="/home/container/gost -C /home/container/conf/gost.yml"
    fi

    create_default_config

    LOCAL_VERSION=$(get_current_version)

    if [ "$VERSION" == "latest" ]; then
        
        REMOTE_VERSION=$(get_latest_version)

        if [ "$REMOTE_VERSION" != "v$LOCAL_VERSION" ]; then
            echo "New version available: $REMOTE_VERSION. Downloading..."
            download_ntfy $REMOTE_VERSION $ARCH
        else
            echo "The latest version is already installed. Continuing..."
        fi
    else
        
        if [[ "v$VERSION" != "v$LOCAL_VERSION" ]]; then
            echo "Requested version $VERSION is not installed. Downloading..."
            download_ntfy v$VERSION $ARCH
        else
            echo "Requested version $VERSION is already installed. Continuing..."
        fi

    fi

    supervisord -c /etc/supervisor.d/services.ini
}

main

