#!/bin/bash

# Pterodactyl Custom ENV Variables:

# GIT_REPO_URL - The Git repository URL to clone if the /home/container/app directory does not exist.
# GIT_BRANCH - The branch to clone from the Git repository.
# GIT_USERNAME - The username for Git authentication (if needed).
# GIT_ACCESS_TOKEN - The access token or password for Git authentication (if needed).

# INSTALL_NODE_PACKAGES - Additional packages to install via apt-get.
# UNINSTALL_NODE_PACKAGES - Additional packages to uninstall via apt-get.

# SUPERVISOR_API_KEY - The API key for authenticating requests to the supervisor API.

function install_or_update_bun {
    if [ ! -f /home/container/.bun/bin/bun ]; then
        echo "Bun not found. Installing Bun..."
        curl -fsSL https://bun.sh/install | bash
    else
        echo "Bun found. Updating Bun to the latest version..."
        yes | /home/container/.bun/bin/bun upgrade
    fi

    if [ $? -ne 0 ]; then
        echo "Bun installation or update failed!"
        exit 1
    fi

    export BUN_INSTALL="/home/container/.bun"
    export PATH="$BUN_INSTALL/bin:$PATH"
}

function create_directories {
    mkdir -p /home/container/.app/
    mkdir -p /home/container/.app/tmp
    mkdir -p /home/container/.app/instance/1/
    mkdir -p /home/container/.app/instance/2/
}


function clone_git_repo_if_needed {

    if [ -d "/home/container/.app/git-repo/.git" ]; then
        echo "Git repository already cloned."
        return
    fi

    # clean up possible existing directory
    rm -rf /home/container/.app/git-repo    

    local FULL_GIT_REPO_URL="${GIT_REPO_URL}"

    if [[ ${GIT_REPO_URL} != *.git ]]; then
        FULL_GIT_REPO_URL="${GIT_REPO_URL}.git"
    fi

    if [ -n "${GIT_USERNAME}" ] && [ -n "${GIT_ACCESS_TOKEN}" ]; then
        FULL_GIT_REPO_URL="https://${GIT_USERNAME}:${GIT_ACCESS_TOKEN}@$(echo -e ${GIT_REPO_URL} | cut -d/ -f3-)"
    fi

    if [ ! -d "/home/container/app" ]; then
        echo "Cloning Git repository from ${GIT_REPO_URL}..."
        if [ -n "${GIT_BRANCH}" ]; then
            git clone --single-branch --branch "${GIT_BRANCH}" "${FULL_GIT_REPO_URL}" /home/container/.app/git-repo
        else
            git clone "${FULL_GIT_REPO_URL}" /home/container/.app/git-repo
        fi

    fi
}


function main {

    echo "Starting..."

    cd /home/container

    install_or_update_bun
    create_directories

    clone_git_repo_if_needed

    export SUPERVISOR_PROXY_LISTEN="0.0.0.0:${SERVER_PORT}"

    # Extract Startup CMD
    STARTUP_CMD=$(echo ${STARTUP} | sed -e 's/{{/${/g' -e 's/}}/}/g')

    eval ${STARTUP_CMD}

    # STARTUP_CMD is normaly: /usr/bin/supervisor
}

main