FROM ubuntu:22.04 as UBUNTU

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y \
    curl \
    gnupg \
    lsb-release \
    ca-certificates \
    gcc \
    libssl-dev \
    pkg-config

# Install Docker CLI
RUN mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
RUN echo \
      "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
      $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
RUN apt-get update && apt-get install -y docker-ce-cli

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
RUN cargo install --force cargo-make
