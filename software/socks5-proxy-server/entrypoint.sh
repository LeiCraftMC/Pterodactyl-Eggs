#!/bin/bash

function check_cpu_arch {
    local arch=$(uname -m)
    if [ "$arch" = "x86_64" ]; then
        declare -g ARCH="linux-amd64"
    elif [ "$arch" = "aarch64" ]; then
        declare -g ARCH="linux-arm64"
    else
        echo "Unsupported architecture: $arch"
        exit 1
    fi
}

function extract_env_bool {
    local var_name=$1
    if [[ "${!var_name}" == "1" ]]; then
        eval "$var_name=true"
    else
        eval "$var_name=false"
    fi
}

function get_latest_version {
    local include_prereleases=$1
    local api_url="https://api.github.com/repos/ariadata/go-socks5-proxy/releases"

    # Fetch releases from GitHub API
    local releases_json=$(curl -s "$api_url")

    if [ "$include_prereleases" == "true" ]; then
        # Include prereleases
        echo "$releases_json" | jq -r '.[0].tag_name'
    else
        # Exclude prereleases, find the first stable release
        echo "$releases_json" | jq -r '[.[] | select(.prerelease == false)][0].tag_name'
    fi
}

function get_current_version {
    # output is in format socks5-proxy-server version 1.0.0
    local version=$(echo $(/home/container/.bin/socks5-proxy-server --version 2>/dev/null) | awk '{print $4}')
    echo ${version#v}
}

function download_app {
    local version=$1
    local arch=$2
    local name="go-socks5-proxy-${arch}"
    local url="https://github.com/ariadata/go-socks5-proxy/releases/download/${version}/${name}.tar.gz"

    echo "Downloading socks5-proxy-server version $version for architecture $arch..."

    http_response_code="$(curl --write-out '%{http_code}' -sL -o $name.tar.gz "$url")"

    if [ "$http_response_code" != "200" ]; then
        echo "Failed to download socks5-proxy-server binary. HTTP response code: $http_response_code"
        rm -f 
        exit 1
    fi

    echo "Download complete. Extracting..."
    tar zxvf "${name}.tar.gz"
    if [ $? -ne 0 ]; then
        echo "Failed to extract socks5-proxy-server binary."
        exit 1
    fi
    
    mv /home/container/${name} /home/container/.bin/socks5-proxy-server
    chmod +x /home/container/.bin/socks5-proxy-server

    rm "${name}.tar.gz"
    rm -rf $name

    echo "socks5-proxy-server $version downloaded successfully."
}

function create_directories {
    mkdir -p /home/container/.bin
}

function create_conf_files {

    if [ ! -f /home/container/users.conf ]; then
        echo "Creating users config file..."
        cat <<EOF > /home/container/users.conf
# SOCKS5 Proxy User Configuration
# Format: username:password
# Lines starting with # are comments

EOF
    fi
    
}


function main {

    echo "Starting..."

    cd /home/container

    check_cpu_arch

    # Extract Startup CMD
    STARTUP_CMD=$(echo ${STARTUP} | sed -e 's/{{/${/g' -e 's/}}/}/g')
    extract_env_bool EXPERIMENTAL
    
    create_directories
    create_conf_files

    LOCAL_VERSION=$(get_current_version)

    if [ "$VERSION" == "latest" ]; then
        
        REMOTE_VERSION=$(get_latest_version $EXPERIMENTAL)

        if [ "$REMOTE_VERSION" != "$LOCAL_VERSION" ]; then
            echo "New version available: $REMOTE_VERSION. Downloading..."
            download_app $REMOTE_VERSION $ARCH
        else
            echo "The latest version is already installed. Continuing..."
        fi
    else
        
        if [[ "$VERSION" != "$LOCAL_VERSION" ]]; then
            echo "Requested version $VERSION is not installed. Downloading..."
            download_app v$VERSION $ARCH
        else
            echo "Requested version $VERSION is already installed. Continuing..."
        fi

    fi

    eval ${STARTUP_CMD}
}

main
