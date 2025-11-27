#/bin/bash

if [ -d "/home/container/.app/git-repo/.git" ]; then
    echo "Pulling latest changes from Git repository..."
    cd /home/container/.app/git-repo
    git pull
    cd /home/container
else 
    echo "No Git repository found to pull changes from."
    exit 1
fi


