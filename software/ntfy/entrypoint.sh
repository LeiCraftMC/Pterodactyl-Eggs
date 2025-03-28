#!/bin/bash

wget https://github.com/binwiederhier/ntfy/releases/download/v2.11.0/ntfy_2.11.0_linux_amd64.tar.gz
tar zxvf ntfy_2.11.0_linux_amd64.tar.gz
cp -a ntfy_2.11.0_linux_amd64/ntfy /usr/local/bin/ntfy
mkdir /etc/ntfy && cp ntfy_2.11.0_linux_amd64/{client,server}/*.yml /etc/ntfy
ntfy serve
