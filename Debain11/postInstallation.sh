#!/bin/bash

function setupRootPW() {
    while true; do
        echo "Enter the new root password (at least 8 characters):"
        read -s new_password  # Read input silently (without displaying characters)

        if [ ${#new_password} -lt 8 ]; then
            echo "Password must be at least 8 characters long. Please try again."
            continue
        fi

        echo "Confirm the new root password:"
        read -s confirm_password

        if [ -z "$new_password" ] || [ "$new_password" != "$confirm_password" ]; then
            echo "Passwords do not match or are empty. Please try again."
            continue
        fi

        # Use `sudo` to change the root password
        echo -e "$new_password\n$new_password" | sudo passwd root

        if [ $? -eq 0 ]; then
            echo "Root password successfully changed."
            break
        else
            echo "Failed to change root password."
            return 1  # Return an error status
        fi
    done
}


function setupSSH() {

    apt install dropbear -y

    local dropbear_file="/etc/default/dropbear"

    # Check if the file exists
    if [ ! -f "$dropbear_file" ]; then
        echo "Error: $dropbear_file not found."
        return 1
    fi

    # Check if the line exists in the file
    if grep -q '^DROPBEAR_PORT=' "$dropbear_file"; then
        # Replace the line in the file
        sed -i "s/^DROPBEAR_PORT=.*/DROPBEAR_PORT=${SERVER_PORT}/" "$dropbear_file"
        echo "DROPBEAR_PORT in $dropbear_file replaced with $SERVER_PORT."
    else
        echo "Error: DROPBEAR_PORT not found in $dropbear_file."
        return 1
    fi

}


if [ ! -e "/.postInstallationMade" ]; then

    setupRootPW

    setupSSH

    touch "/.postInstallationMade"
fi