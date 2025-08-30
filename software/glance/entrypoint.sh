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
    local api_url="https://api.github.com/repos/glanceapp/glance/releases"

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
    local version=$(./glance -v 2>/dev/null)
    echo $version | cut -d ' ' -f 2
}

function download_app {
    local version=$1
    local arch=$2
    local name="glance-${arch}"
    local url="https://github.com/glanceapp/glance/releases/download/${version}/${name}.tar.gz"

    echo "Downloading glance version $version for architecture $arch..."

    http_response_code="$(curl --write-out '%{http_code}' -sL -o $name.tar.gz "$url")"

    if [ "$http_response_code" != "200" ]; then
        echo "Failed to download glance binary. HTTP response code: $http_response_code"
        exit 1
    fi

    echo "Download complete. Extracting..."
    tar zxvf "${name}.tar.gz"
    if [ $? -ne 0 ]; then
        echo "Failed to extract glance binary."
        exit 1
    fi
    
    chmod +x /home/container/glance

    rm "${name}.tar.gz"
    rm -rf $name

    echo "glance $version downloaded successfully."
}

function create_directories {
    mkdir -p /home/container/conf
    mkdir -p /home/container/samples
}

function download_base_conf {
    local version=$1
    local url="https://raw.githubusercontent.com/glanceapp/glance/refs/tags/${version}/docs/glance.yml"

    if [[ ! -f /home/container/samples/glance.yml ]]; then
        curl -o /home/container/samples/glance.yml "$url"
    fi

    if [[ -f /home/container/conf/glance.yml ]]; then
        echo "Base config file already exists. Skipping download."
    else
        cp /home/container/samples/glance.yml /home/container/conf/glance.yml
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

    LOCAL_VERSION=$(get_current_version)

    if [ "$VERSION" == "latest" ]; then
        
        REMOTE_VERSION=$(get_latest_version $EXPERIMENTAL)

        if [ "$REMOTE_VERSION" != "$LOCAL_VERSION" ]; then
            echo "New version available: $REMOTE_VERSION. Downloading..."
            download_app $REMOTE_VERSION $ARCH
            download_base_conf $REMOTE_VERSION
        else
            echo "The latest version is already installed. Continuing..."
        fi
    else
        
        if [[ "v$VERSION" != "$LOCAL_VERSION" ]]; then
            echo "Requested version $VERSION is not installed. Downloading..."
            download_app v$VERSION $ARCH
            download_base_conf v$VERSION
        else
            echo "Requested version $VERSION is already installed. Continuing..."
        fi

    fi

    eval ${STARTUP_CMD}
}

main
