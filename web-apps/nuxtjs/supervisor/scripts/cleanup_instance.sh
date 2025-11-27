#!/bin/bash

INSTANCE_NUMBER=$1

if [ ${INSTANCE_NUMBER} != "1" ] && [ ${INSTANCE_NUMBER} != "2" ]; then
    echo "Invalid instance number provided. Must be 1 or 2."
    exit 1
fi

rm -rf /home/container/.app/instance/${INSTANCE_NUMBER}/*
