FROM --platform=amd64 debian:12-slim

WORKDIR /usr/local/webol
RUN ls -la
COPY target/armv7-unknown-linux-gnueabihf/release/webol /usr/local/bin/webol

EXPOSE 7229
CMD ["webol"]
