#!/bin/bash
sleep 2
export HOME=/home/container
cd /home/container
MODIFIED_STARTUP=`eval echo $(echo ${STARTUP} | sed -e 's/{{/${/g' -e 's/}}/}/g')`

# Make internal Docker IP address available to processes.
export INTERNAL_IP=`ip route get 1 | awk '{print $NF;exit}'`

if [ "$EXPERIMENTAL" -ne 1 ] || [ ! -e "/home/container/startup.sh" ]; then
	curl -Ls -H 'Pragma: no-cache' \
             -H 'Cache-Control: no-cache, no-store' \
             https://raw.githubusercontent.com/LeiCraft/Pterodactyl-Eggs/main/Debain11/startup.sh -o startup.sh
    chmod +x ./startup.sh
fi

# Run the VPS Installer
bash ./startup.sh
