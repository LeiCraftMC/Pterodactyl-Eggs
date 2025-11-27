#!/bin/bash

INSTANCE_NUMBER=$1

if [ ${INSTANCE_NUMBER} != "1" ] && [ ${INSTANCE_NUMBER} != "2" ]; then
    echo "Invalid instance number provided. Must be 1 or 2."
    exit 1
fi

if [ ! -d "/home/container/.app/git-repo/.output" ]; then
    echo "No build found to move. Please create a build first."
    exit 1
fi

rm -rf /home/container/.app/instance/${INSTANCE_NUMBER}/*

cp -r /home/container/.app/git-repo/.output/* /home/container/.app/instance/${INSTANCE_NUMBER}/

rm -rf /home/container/.app/git-repo/.output
rm -rf /home/container/.app/git-repo/.nuxt
rm -rf /home/container/.app/git-repo/node_modules

echo "Build moved to instance ${INSTANCE_NUMBER} successfully."