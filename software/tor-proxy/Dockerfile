FROM alpine

RUN apk add --no-cache --update iptables curl ca-certificates tar bash jq \
    && adduser --disabled-password --home /home/container container

RUN echo '@edge https://dl-cdn.alpinelinux.org/alpine/edge/community' >> /etc/apk/repositories && \
    echo '@edge https://dl-cdn.alpinelinux.org/alpine/edge/testing'   >> /etc/apk/repositories && \
    apk -U upgrade && \
    apk add --no-cache iptables supervisor curl ca-certificates tar bash jq && \
    apk -v add tor@edge obfs4proxy@edge && \
    rm -rf /var/cache/apk/*

COPY --chown=container:container assets/services.ini /etc/supervisor.d/
COPY --chown=container:container assets/torrc /etc/tor/torrc

RUN mkdir -p /home/container/tor && \
    mv /var/lib/tor /home/container/tor/data && \
    chown -R container:container /home/container/tor && \
    chmod 700 /home/container/tor && \
    mkdir -p /etc/supervisor.d

USER container
ENV  USER=container HOME=/home/container
    
WORKDIR /home/container
    
COPY ./entrypoint.sh /entrypoint.sh
    
CMD ["/bin/bash", "/entrypoint.sh"]
