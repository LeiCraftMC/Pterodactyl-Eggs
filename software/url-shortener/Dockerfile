FROM node:22-alpine

RUN apk add --no-cache --update tzdata curl ca-certificates tar bash jq \
    && adduser --disabled-password --home /home/container container

USER container
ENV  USER=container HOME=/home/container
    
WORKDIR /home/container
    
COPY ./entrypoint.sh /entrypoint.sh
COPY ./theme /home/container/theme

ENV NODE_ENV=production

ENV DB_FILENAME="/home/container/data/db.sqlite"

CMD ["/bin/bash", "/entrypoint.sh"]
