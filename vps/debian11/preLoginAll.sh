#!/bin/bash

export LHOSTNAME=$new_hostname
export HOSTNAME=$new_hostname

function setupBashrc() {
    local file="$1"
    # Check if .bashrc exists for the user
    if [ -f "$file" ]; then
        # Replace \h with new_hostname in the .bashrc file
        sed -i 's/\\h/${LHOSTNAME}/g' "$file"

        local preLoginUser_runLine="bash /home/container/preLoginUser.sh"

        # Check if the line already exists in the fil
        if ! grep -Fxq "$preLoginUser_runLine" "$file"; then
            # Append the line to the end of the file
            echo "$line" >> "$file"
        fi

    fi
}

function setupUserHostname() {

    # Loop through each user's home directory
    for user_home in /home/*; do
        if [ -d "$user_home" ]; then
            setupBashrc "$user_home/.bashrc"
        fi
    done

    # Also update .bashrc for the root user if exists
    setupBashrc "/root/.bashrc"

}

setupUserHostname
