#!/bin/bash

if [ -d "/home/container/.app/git-repo/.git" ]; then

    echo "Cleaning up build artifacts from Git repository..."
    cd /home/container/.app/git-repo

    rm -rf node_modules
    rm -rf .nuxt
    rm -rf .output

    cd /home/container

else 
    echo "No Git repository found to clean up build artifacts from."
    exit 1
fi