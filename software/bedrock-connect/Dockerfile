FROM openjdk:17-jdk-alpine

RUN apk -U upgrade && apk add --no-cache --update tzdata curl ca-certificates supervisor tar bash jq rsync \
    && adduser --disabled-password --home /home/container container

COPY --chown=container:container assets/services.ini /etc/supervisor.d/

USER container
ENV  USER=container HOME=/home/container
    
WORKDIR /home/container
    
COPY ./entrypoint.sh /entrypoint.sh
    
CMD ["/bin/bash", "/entrypoint.sh"]
