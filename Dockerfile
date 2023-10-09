FROM --platform=amd64 debian:bullseye-slim

WORKDIR /usr/local/webol
COPY target/armv7-unknown-linux-gnueabihf/release/webol /usr/local/bin/webol

EXPOSE 7229
CMD ["webol"]