FROM ubuntu:22.04 as UBUNTU

RUN apt update -y && apt upgrade -y && apt install curl -y

# Add rustup and default nightly toolchain
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path --profile default --default-toolchain nightly -c rustfmt && \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME && \
    rustup --version; \
    cargo --version; \
    rustc --version

## Packages needed for building services
RUN apt install -y gcc libssl-dev pkg-config
RUN cargo install --force cargo-make
