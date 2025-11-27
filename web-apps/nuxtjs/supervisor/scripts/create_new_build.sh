#!/bin/bash

if [ -d "/home/container/.app/git-repo/.git" ]; then

    echo "Creating new build from Git repository..."
    cd /home/container/.app/git-repo

    /usr/local/share/supervisor/scripts/cleanup_build_artifacts.sh

    bun install
    bun run build

    cd /home/container

else 
    echo "No Git repository found to build from."
    exit 1
fi