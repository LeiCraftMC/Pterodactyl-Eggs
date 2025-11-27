FROM debian:stable-slim

RUN apt-get update && apt-get install -y tzdata curl ca-certificates git tar bash jq unzip \
    && adduser --disabled-password --home /home/container container

COPY ./supervisor/target/release/supervisor /usr/bin/supervisor
RUN chmod +x /usr/bin/supervisor

USER container
ENV  USER=container HOME=/home/container
    
WORKDIR /home/container
    
COPY ./entrypoint.sh /entrypoint.sh
COPY ./supervisor/scripts /usr/local/share/supervisor/scripts
# RUN chmod +x /entrypoint.sh /usr/local/share/supervisor/scripts/*

CMD ["/bin/bash", "/entrypoint.sh"]
