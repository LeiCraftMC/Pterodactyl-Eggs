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

    local repo_owner=$2
    local repo_name=$3

    local api_url="https://api.github.com/repos/${repo_owner}/${repo_name}/releases"

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
    local subpath=$1

    cat /home/container/${subpath}/.version 2>/dev/null || echo "0.0.0"
}

function download_bedrock_connect {
    local version=$1
    local arch=$2
    local name="BedrockConnect-1.0-SNAPSHOT.jar"
    local url="https://github.com/Pugmatt/BedrockConnect/releases/download/${version}/${name}"

    echo "Downloading bedrock-connect version $version for architecture $arch..."

    http_response_code="$(curl --write-out '%{http_code}' -sL -o $name "$url")"

    if [ "$http_response_code" != "200" ]; then
        echo "Failed to download bedrock-connect. HTTP response code: $http_response_code"
        exit 1
    fi

    cp -a ${name} /home/container/bedrock-connect/BedrockConnect.jar
    chmod +x /home/container/bedrock-connect/BedrockConnect.jar

    rm -rf $name

    echo -n "$(echo $version | cut -c2-)" > /home/container/bedrock-connect/.version

    echo "bedrock-connect $version downloaded successfully."
}

function download_mcxboxbroadcast {
    local version=$1
    local arch=$2
    local name="MCXboxBroadcastStandalone.jar"
    local url="https://github.com/MCXboxBroadcast/Broadcaster/releases/download/${version}/${name}"

    echo "Downloading mcxboxbroadcast version $version for architecture $arch..."

    http_response_code="$(curl --write-out '%{http_code}' -sL -o $name "$url")"

    if [ "$http_response_code" != "200" ]; then
        echo "Failed to download mcxboxbroadcast. HTTP response code: $http_response_code"
        exit 1
    fi

    cp -a ${name} /home/container/mcxboxbroadcast/MCXboxBroadcast.jar
    chmod +x /home/container/mcxboxbroadcast/MCXboxBroadcast.jar

    rm -rf $name

    echo -n "$(echo $version | cut -c2-)" > /home/container/mcxboxbroadcast/.version

    echo "mcxboxbroadcast $version downloaded successfully."
}

function create_directories {
    mkdir -p /home/container/bedrock-connect
    mkdir -p /home/container/mcxboxbroadcast
}


function update_if_needed_bedrock_connect {

    local LOCAL_VERSION=$(get_current_version "bedrock-connect")
    
    if [ "$BEDROCK_CONNECT_VERSION" == "latest" ]; then
        
        local REMOTE_VERSION=$(get_latest_version false "Pugmatt" "BedrockConnect")

        if [ "$REMOTE_VERSION" != "$LOCAL_VERSION" ]; then
            echo "New version available: $REMOTE_VERSION. Downloading..."
            download_bedrock_connect $REMOTE_VERSION $ARCH
        else
            echo "The latest version is already installed. Continuing..."
        fi
    else
        
        if [[ "$BEDROCK_CONNECT_VERSION" != "$LOCAL_VERSION" ]]; then
            echo "Requested version $BEDROCK_CONNECT_VERSION is not installed. Downloading..."
            download_bedrock_connect $BEDROCK_CONNECT_VERSION $ARCH
        else
            echo "Requested version $BEDROCK_CONNECT_VERSION is already installed. Continuing..."
        fi

    fi

}

function update_if_needed_mcxboxbroadcast {

    local LOCAL_VERSION=$(get_current_version "mcxboxbroadcast")
    
    if [ "$MCXBOXBROADCAST_VERSION" == "latest" ]; then
        
        local REMOTE_VERSION=$(get_latest_version false "MCXboxBroadcast" "Broadcaster")

        if [ "$REMOTE_VERSION" != "$LOCAL_VERSION" ]; then
            echo "New version available: $REMOTE_VERSION. Downloading..."
            download_mcxboxbroadcast $REMOTE_VERSION $ARCH
        else
            echo "The latest version is already installed. Continuing..."
        fi
    else
        
        if [[ "$MCXBOXBROADCAST_VERSION" != "$LOCAL_VERSION" ]]; then
            echo "Requested version $MCXBOXBROADCAST_VERSION is not installed. Downloading..."
            download_mcxboxbroadcast $MCXBOXBROADCAST_VERSION $ARCH
        else
            echo "Requested version $MCXBOXBROADCAST_VERSION is already installed. Continuing..."
        fi

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

    update_if_needed_bedrock_connect
    update_if_needed_mcxboxbroadcast

    supervisord -c /etc/supervisor.d/services.ini

    eval ${STARTUP_CMD}

    # STARTUP_CMD should be "supervisord -c /etc/supervisor.d/services.ini"
    # supervisorctl
}

main
