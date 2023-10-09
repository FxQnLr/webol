FROM --platform=arm64 debian:12-slim

WORKDIR /usr/local/webol
COPY ./target/armv7-unknown-linux-gnueabihf/release/webol /usr/local/bin/webol

EXPOSE 7229
CMD ["webol"]
