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
    local api_url="https://api.github.com/repos/pocketbase/pocketbase/releases"

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
    # output is in format pocketbase version 0.29.2
    local version=$(./.bin/pocketbase -v 2>/dev/null)
    echo $version | awk '{print $3}'
}

function download_app {
    local version=$1
    local version_without_v=${version#v}
    local arch=$2
    local name="pocketbase_${version_without_v}_${arch}"
    local url="https://github.com/pocketbase/pocketbase/releases/download/${version}/${name}.zip"

    echo "Downloading pocketbase version $version for architecture $arch..."

    http_response_code="$(curl --write-out '%{http_code}' -sL -o $name.zip "$url")"

    if [ "$http_response_code" != "200" ]; then
        echo "Failed to download pocketbase binary. HTTP response code: $http_response_code"
        rm -f $name.zip
        exit 1
    fi

    echo "Download complete. Extracting..."

    unzip -o $name.zip -d tmp-$name
    if [ $? -ne 0 ]; then
        echo "Failed to extract pocketbase binary."
        exit 1
    fi
    
    mv tmp-$name/pocketbase /home/container/.bin/pocketbase
    chmod +x /home/container/.bin/pocketbase

    rm "${name}.zip"
    rm -rf tmp-$name

    echo "pocketbase $version downloaded successfully."
}

function create_directories {
    mkdir -p /home/container/.bin
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
