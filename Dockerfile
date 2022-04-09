FROM arm64v8/rust:1.59-alpine3.15

# Adding the nightly toolchain
RUN rustup toolchain install nightly-2022-03-23 -t aarch64-unknown-linux-musl && \
    rustup default nightly-2022-03-23 && \
    rustup component add rustfmt


# Packages needed for building services
RUN apk update
RUN apk add --no-cache make python3 musl-dev openssl-dev protoc docker docker-cli bash aws-cli

# Volume with source code must be mounted at /uc/src
